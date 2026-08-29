# SPEC.md — Terminal File Manager (`tfm`)

The contract. When this file and any other doc disagree, **this file wins**.
`CLAUDE.md` says *how* to build; this says *what* is built.

Version: v1 (Linux x86_64 only)

---

## 1. Product definition

`tfm` is a local-first Linux file manager. A Rust/Tauri 2 backend owns every
filesystem operation and all OS integration. A React + TypeScript + Vite +
Tailwind frontend renders a dark, terminal-aesthetic, keyboard-first UI inside
the webview.

The frontend is a **view over backend state**. It holds no filesystem
capability of its own: no `fs` access from JS, no shelling out, no path
guessing. Every read, every mutation, every launch is a Tauri `#[command]`.

### Goals (v1)
1. Browse a directory tree quickly with the keyboard alone.
2. Inspect an entry's real metadata (mode, owner, group, times, size, type).
3. Perform the five ordinary mutations — create, rename, copy, move, delete —
   safely, with delete routed to Trash.
4. Open files and folders in the user's configured default application.
5. Stay live: reflect external filesystem changes without a manual refresh.

### Non-Goals (v1)
Do not build these without an explicit request. Note the idea and move on.
- Dual-pane / Norton-style layout
- Image or document thumbnails, previews beyond a plain-text head
- Archive creation or browsing (zip, tar, …)
- Network mounts, SFTP, cloud remotes, GVFS
- Tabs, bookmarks, saved sessions
- File content editing
- Batch rename / regex rename
- Cross-platform support (macOS, Windows)
- Elevated-privilege operations (`sudo`, polkit, mounting)
- Automatic recursive directory sizes in the list view (on-demand, one
  directory at a time, is supported — §4.6)

---

## 2. Data types

These shapes are the contract. Rust structs use `#[serde(rename_all =
"camelCase")]`; the TypeScript in `src/types.ts` must mirror them exactly.
**Changing a type here requires changing `types.ts` and `api.ts` in the same
commit.**

### 2.1 `EntryKind`

```ts
type EntryKind = "file" | "directory" | "symlink" | "other";
```

- `"symlink"` is reported for the link itself, regardless of target type.
  The resolved target's kind is carried separately (`symlinkTargetKind`).
- `"other"` covers sockets, FIFOs, block and character devices.

### 2.2 `DirEntry`

The cheap, list-view shape. One `lstat` per entry, no target resolution beyond
a single `stat` for symlinks. Never triggers recursive work.

```ts
interface DirEntry {
  name: string;              // file name only, not a path
  path: string;              // absolute, canonicalized parent + name
  kind: EntryKind;
  size: number;              // bytes; for directories, the dir inode size, NOT recursive
  modifiedMs: number | null; // Unix epoch milliseconds; null if unavailable
  isHidden: boolean;         // name starts with '.'
  isSymlink: boolean;
  symlinkTargetKind: EntryKind | null; // null when not a symlink or target unreadable
  isBrokenSymlink: boolean;  // symlink whose target cannot be stat'd
  readable: boolean;         // effective read access for the invoking user
}
```

### 2.3 `EntryInfo`

The expensive, details-pane shape. Requested for one entry at a time.

```ts
interface EntryInfo {
  name: string;
  path: string;
  kind: EntryKind;
  size: number;
  sizeHuman: string;         // e.g. "1.4 KiB", "0 B" — see §6.2
  modifiedMs: number | null;
  accessedMs: number | null;
  createdMs: number | null;  // often null on Linux filesystems; that is expected
  mode: number;              // raw st_mode permission bits, 0o777 masked
  modeString: string;        // e.g. "drwxr-xr-x" — see §6.1
  owner: string;             // username, or the numeric uid as a string on lookup failure
  group: string;             // group name, or the numeric gid as a string
  uid: number;
  gid: number;
  nlink: number;
  inode: number;
  isHidden: boolean;
  isSymlink: boolean;
  symlinkTarget: string | null;   // raw link text, not resolved
  symlinkResolved: string | null; // canonicalized target, null if broken
  isBrokenSymlink: boolean;
  mimeType: string | null;   // content sniff via `infer`, null when undetermined
  childCount: number | null; // directories only: immediate children; null otherwise or on EACCES
}
```

`childCount` is one non-recursive `read_dir` count, and `size` for a directory
is its own inode size — neither is a recursive measurement. Recursive size is a
separate, explicitly requested operation; see §4.6.

### 2.4 `AppError`

Every command returns `Result<T, AppError>`. Serialized to the frontend as a
two-field object — never as a bare string.

```ts
interface AppError {
  kind: ErrorKind;
  message: string;   // human-readable, safe to display; includes the offending path
}

type ErrorKind =
  | "NotFound"
  | "PermissionDenied"
  | "AlreadyExists"
  | "NotADirectory"
  | "IsADirectory"
  | "CrossDevice"
  | "InvalidPath"
  | "DirectoryNotEmpty"
  | "WouldRecurse"      // move/copy of a directory into its own subtree
  | "Cancelled"         // a long-running op was cancelled by the user
  | "Unsupported"
  | "Io";               // last resort; message carries the raw io::Error text
```

Mapping rules (backend, `error.rs`):

| `std::io::ErrorKind` / condition            | `ErrorKind`         |
|---------------------------------------------|---------------------|
| `NotFound`                                    | `NotFound`          |
| `PermissionDenied`                            | `PermissionDenied`  |
| `AlreadyExists`                               | `AlreadyExists`     |
| `raw_os_error == EXDEV (18)`                  | `CrossDevice`       |
| `raw_os_error == ENOTDIR (20)`                | `NotADirectory`     |
| `raw_os_error == EISDIR (21)`                 | `IsADirectory`      |
| `raw_os_error == ENOTEMPTY (39)`              | `DirectoryNotEmpty` |
| path validation failure (§3.2)                | `InvalidPath`       |
| destination inside source subtree             | `WouldRecurse`      |
| user cancelled a long-running op (§4.6)       | `Cancelled`         |
| non-regular, non-dir target for an op         | `Unsupported`       |
| anything else                                 | `Io`                |

`Io` is a fallback, not a default. Adding a command means checking whether a
new distinct kind is warranted.

### 2.5 Sorting and filtering

Sort is applied **in the backend** so that list order is stable and identical
across a refresh and a watch-driven reload.

```ts
type SortKey = "name" | "size" | "modified" | "kind";
interface ListOptions {
  sort: SortKey;
  descending: boolean;
  showHidden: boolean;
}
```

Ordering rules, in priority order:
1. Directories before files before `other`, always — regardless of `sort`.
   (Symlinks sort with their `symlinkTargetKind`; broken symlinks sort as files.)
2. Then by `sort` key. `name` is case-insensitive, then case-sensitive as a
   tiebreaker, using byte order for non-ASCII — no locale collation in v1.
3. `descending` reverses step 2 only, never step 1.

---

## 3. Backend API

All commands live in `src-tauri/src/commands.rs`, are thin, and delegate to
`read` / `ops` / `open` / `watch` / `size`. Argument names are camelCase on the
wire.

### 3.1 Command table

| Command | Args | Returns | Notable errors |
|---|---|---|---|
| `list_dir` | `path: String, options: ListOptions` | `Vec<DirEntry>` | `NotFound`, `PermissionDenied`, `NotADirectory` |
| `entry_info` | `path: String` | `EntryInfo` | `NotFound`, `PermissionDenied` |
| `directory_size` | `path: String` | `DirSize` | `NotFound`, `PermissionDenied`, `NotADirectory`, `Cancelled` |
| `cancel_directory_size` | `path: String` | `()` | — |
| `read_text_head` | `path: String, maxBytes: u64` | `TextHead` | `NotFound`, `PermissionDenied`, `IsADirectory`, `Unsupported` |
| `home_dir` | — | `String` | `Io` |
| `parent_dir` | `path: String` | `Option<String>` | `InvalidPath` |
| `places` | — | `Vec<Place>` | — |
| `create_dir` | `parent: String, name: String` | `String` (new path) | `AlreadyExists`, `PermissionDenied`, `InvalidPath` |
| `create_file` | `parent: String, name: String` | `String` (new path) | `AlreadyExists`, `PermissionDenied`, `InvalidPath` |
| `rename_entry` | `path: String, newName: String` | `String` (new path) | `AlreadyExists`, `NotFound`, `PermissionDenied`, `InvalidPath` |
| `copy_entries` | `sources: Vec<String>, destDir: String, overwrite: bool` | `OpOutcome` | `AlreadyExists`, `WouldRecurse`, `PermissionDenied` |
| `move_entries` | `sources: Vec<String>, destDir: String, overwrite: bool` | `OpOutcome` | `AlreadyExists`, `WouldRecurse`, `CrossDevice` (handled, see §4.3) |
| `trash_entries` | `paths: Vec<String>` | `OpOutcome` | `NotFound`, `PermissionDenied`, `Unsupported` |
| `open_path` | `path: String` | `()` | `NotFound`, `PermissionDenied`, `Unsupported` |
| `reveal_in_terminal` | `path: String` | `()` | `Unsupported`, `NotFound` |
| `watch_dir` | `path: String` | `()` | `NotFound`, `PermissionDenied` |
| `unwatch_all` | — | `()` | — |

Supporting shapes:

```ts
interface DirSize {
  path: string;         // the canonicalized directory measured
  bytes: number;        // sum of apparent file sizes in the subtree
  sizeHuman: string;    // formatted per §6.2
  fileCount: number;
  dirCount: number;     // excludes the measured directory itself
  complete: boolean;    // false when subtrees were skipped (see errorCount)
  errorCount: number;   // entries skipped, almost always EACCES
  measuredAtMs: number; // when the walk finished — this is a snapshot, not live
}

interface TextHead {
  content: string;    // lossy UTF-8 decode of the first maxBytes
  truncated: boolean;
  isBinary: boolean;  // NUL byte found in the sampled window
}

interface Place {              // sidebar shortcuts
  label: string;               // "Home", "Documents", "Downloads", …
  path: string;
}

interface OpOutcome {
  succeeded: string[];         // source paths that completed
  failed: OpFailure[];         // per-entry failures; the batch does NOT abort
}

interface OpFailure {
  path: string;
  error: AppError;
}
```

Batch operations (`copy_entries`, `move_entries`, `trash_entries`) are
**partial-success**: one failing source does not roll back or halt the others.
The UI reports what failed. There is no undo in v1 — Trash is the safety net.

### 3.2 Path validation

Applied to *every* path argument before any filesystem action, in `path.rs`:

1. Reject the empty string, any path containing a NUL byte, and any relative
   path → `InvalidPath`.
2. Canonicalize the path (or, for a not-yet-existing target, canonicalize its
   parent and re-join the file name). Canonicalization failure that is not
   `NotFound` → the mapped error.
3. Reject a `name` argument (`create_dir`, `create_file`, `rename_entry`) that
   contains `/`, equals `.` or `..`, is empty, or exceeds 255 bytes →
   `InvalidPath`. `name` is a file name, never a path.
4. Reject any operation whose canonicalized target is `/`, or is `/proc`,
   `/sys`, or `/dev`, or lies within them → `InvalidPath`. These are readable
   in the browser but never a mutation or trash target.
5. For `copy_entries` / `move_entries`: if the canonicalized destination equals
   a source or is a descendant of a source directory → `WouldRecurse`.

Rejecting is always preferable to attempting. "Try it and see what the kernel
says" is not the policy for destructive commands.

### 3.3 Symlink policy

Stated explicitly because implicit symlink behavior is how file managers eat
data.

| Operation | Behavior |
|---|---|
| `list_dir` | `lstat` the entry. One `stat` of the target to fill `symlinkTargetKind` / `isBrokenSymlink`. Never recurses into the target. |
| `entry_info` | Reports both raw link text and the canonicalized target. Metadata (mode, owner, times) is the **link's own**, from `lstat`. |
| Navigate into | Following a symlink to a directory is allowed — it is an explicit user action. The resolved, canonical path becomes the current directory. |
| `copy_entries` | Symlinks are **copied as symlinks** (`symlink` the same raw target). Contents are never dereferenced. A recursive copy never descends through a symlinked directory. |
| `directory_size` | Never descends through a symlinked directory. A symlink contributes the size of the link itself, not its target. |
| `move_entries` | The link itself moves. Target untouched. |
| `trash_entries` | The link itself is trashed. Target untouched. |
| `open_path` | Resolves through the link — that is what the user means by "open". |

---

## 4. Operation semantics

### 4.1 Delete
`trash_entries` routes to the freedesktop Trash via the `trash` crate. v1 has
**no hard-delete command at all.** If one is ever added it must be a separate,
explicitly named command (`delete_permanently`) with its own confirmation
copy and its own UI affordance — never a flag on `trash_entries`.

Requires a frontend confirmation dialog naming the entries and their count.

### 4.2 Copy
- Regular files: byte copy, then apply the source's permission bits. Times are
  not preserved in v1.
- Directories: recursive, depth-first, creating destinations as it goes. Never
  descends through a symlink (§3.3).
- Collision: if the destination path exists and `overwrite` is false → per-entry
  `AlreadyExists` in `OpOutcome.failed`. The backend never silently clobbers.
  `overwrite: true` is only ever set after the frontend has confirmed.
- A source that vanishes mid-operation yields `NotFound` for that entry; the
  rest of the batch continues.

### 4.3 Move
- Attempt `fs::rename` first — the fast path within a filesystem.
- On `EXDEV`, fall back to copy-then-trash-source. The fallback is **not**
  copy-then-`remove`: the source goes to Trash so a partial failure is
  recoverable. If the copy fails, the source is left completely untouched.
- Collision handling matches copy.

### 4.4 Open
`open_path` uses `opener` (xdg-open semantics) and resolves symlinks. Opening a
directory hands it to the desktop's file handler; the app does **not** treat
that as navigation. `reveal_in_terminal` spawns the user's terminal
(`$TERMINAL`, else the first of `x-terminal-emulator`, `gnome-terminal`,
`konsole`, `xfce4-terminal`, `alacritty`, `kitty`, `foot` that resolves on
`PATH`) with cwd set to the directory. If none resolves → `Unsupported`.

### 4.5 Watching
`watch_dir` watches exactly one directory, non-recursively — the current one.
Calling it replaces the previous watch; there is never more than one active
watcher. Events are **debounced 150 ms** in the backend and coalesced into a
single emission.

Event name: `fs://changed`

```ts
interface FsChangedEvent {
  path: string;   // the watched directory
}
```

The payload deliberately carries no diff. The frontend responds by re-issuing
`list_dir` with the current `ListOptions` and reconciling selection by path.
Watch errors (watcher dropped, directory removed) emit:

Event name: `fs://watch-error`, payload `AppError`.

### 4.6 Recursive directory size

Directory sizes are **never computed automatically.** Nothing in the list view
walks a tree; the Size column renders `—` for directories (§5.5). `EntryInfo`
stays cheap. A recursive size is produced only when the user asks for one
specific directory, via `u` on the cursor entry or the details pane's compute
affordance.

`directory_size` is an async command that walks one subtree and returns a
`DirSize`. While it runs:

- Progress is emitted on `dirsize://progress` at most every 100 ms:
  ```ts
  interface DirSizeProgress {
    path: string;      // the directory being measured
    bytes: number;     // running total
    fileCount: number;
  }
  ```
- `cancel_directory_size` sets a cancellation flag the walk checks between
  entries. The in-flight `directory_size` then resolves as `Cancelled`, which
  the UI treats as an ordinary outcome, not an error to display in the status
  bar. `Esc` cancels.
- At most one measurement runs at a time. Issuing a new `directory_size`
  cancels the previous one.

Walk semantics:
- Depth-first over `read_dir`, using `lstat` only — never follows symlinks
  (§3.3).
- Hard links are counted once: `(dev, inode)` pairs seen before are skipped, so
  a tree with hard links does not over-report.
- `bytes` is **apparent size** (the sum of `st_size`), not disk usage. Sparse
  files and filesystem compression mean this can differ from `du`; that is
  expected and the details pane labels it "apparent".
- An unreadable subdirectory is skipped, `errorCount` increments, and
  `complete` becomes false. A partial result is still returned and rendered as
  e.g. `2.1 GiB+ (partial)`. A permission-denied subtree never fails the whole
  measurement.
- Entries that vanish mid-walk are skipped without incrementing `errorCount`.

Caching and staleness. Results are cached in the **frontend** store, keyed by
canonical path. A `DirSize` is a point-in-time snapshot: the watcher is
non-recursive (§4.5), so a change deep inside a measured tree is invisible to
the app. Therefore:
- A cached result is discarded when `fs://changed` fires for the directory
  containing it, on manual refresh, and on navigating away from `cwd`.
- The details pane shows `measuredAtMs` alongside the value once it is older
  than 60 s, so a stale number is never presented as current.
- The backend caches nothing. Every `directory_size` call does the walk.

---

## 5. Frontend contract

### 5.1 Layout

Single pane. Three regions, fixed:

```
┌────────────┬───────────────────────────────┬─────────────┐
│  places    │  path bar                     │             │
│  sidebar   ├───────────────────────────────┤  details    │
│            │                               │  pane       │
│            │  entry list                   │  (selected  │
│            │                               │   entry)    │
│            │                               │             │
├────────────┴───────────────────────────────┴─────────────┤
│  status bar: count · selection · last error               │
└──────────────────────────────────────────────────────────┘
```

### 5.2 Keybindings

All key handling lives in `hooks/useKeyboard.ts` and dispatches to store
actions. No component registers its own listener.

| Key | Action |
|---|---|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `g` / `Home` | Select first |
| `G` / `End` | Select last |
| `Ctrl-d` / `Ctrl-u` | Half-page down / up |
| `l` / `→` / `Enter` | Enter directory, or open file |
| `h` / `←` / `Backspace` | Go to parent |
| `~` | Go home |
| `Space` | Toggle multi-select on the current entry |
| `Esc` | Cancel a running size measurement, else close dialog, else clear filter, else clear selection |
| `.` | Toggle hidden files |
| `/` | Focus the filter input |
| `s` | Cycle sort key |
| `S` | Toggle sort direction |
| `r` | Rename selected |
| `n` | New file |
| `N` | New directory |
| `y` | Yank (mark for copy) |
| `x` | Cut (mark for move) |
| `p` | Paste into current directory |
| `d` | Move to Trash (opens confirmation) |
| `u` | Compute recursive size of the cursor directory (§4.6) |
| `o` | Open with default application |
| `t` | Open terminal here |
| `F5` / `Ctrl-r` | Refresh |
| `?` | Toggle key help overlay |

Every destructive action (`d`, and `p` when it would overwrite) opens a modal
that must be confirmed with `Enter`; `Esc` cancels. The modal names the
affected entries.

### 5.3 Filter
The `/` filter is a **frontend-only, case-insensitive substring match** over
`DirEntry.name` in the already-fetched list. It is not a backend search and
never triggers a new `list_dir`.

### 5.4 State

Zustand store, one slice. Authoritative fields:

```ts
interface AppState {
  cwd: string;
  entries: DirEntry[];
  listOptions: ListOptions;
  cursor: number;              // index into the filtered view
  selected: Set<string>;       // absolute paths
  clipboard: { mode: "copy" | "cut"; paths: string[] } | null;
  filter: string;
  info: EntryInfo | null;      // details pane, for the cursor entry
  dirSizes: Map<string, DirSizeState>;  // §4.6 cache, keyed by canonical path
  dialog: DialogState | null;
  error: AppError | null;      // last error, shown in the status bar
  loading: boolean;
}
```

Rules:
- Selection is stored **by path**, never by index, so a watch-driven reload
  preserves it.
- After a reload, if the cursor's path is gone, the cursor clamps to the same
  index (or the last entry).
- `entry_info` is fetched for the cursor entry on a 120 ms debounce, so holding
  `j` does not issue one command per row.
- `dirSizes` entries are `{ status: "running"; bytes: number; fileCount: number }`
  or `{ status: "done"; value: DirSize }`. Invalidated per §4.6 — never
  persisted across a session.

### 5.5 Size rendering

The list view's Size column shows a formatted size for files and `—` for
directories. It is never the directory's inode size, which is a meaningless
4.0 KiB on every row, and never a recursive walk. `—` reads as "not measured",
which is the truth.

The details pane shows a directory's size as `—  (press u to measure)` until
measured, then the running total while the walk is in flight, then the final
value annotated `apparent` and, if incomplete, `partial — N unreadable`.

### 5.6 Visual language
Monospace throughout. Dark background, one accent color, defined as CSS custom
properties in `theme.css`. No rounded corners, no drop shadows, no gradients,
no transition on anything but opacity. Selection is an inverted-block cursor,
not a rounded highlight. Column layout in the list is fixed-width and aligned
like `ls -l` output.

---

## 6. Formatting rules

Implemented in `src-tauri/src/util.rs`, unit-tested (§7).

### 6.1 Mode string
Ten characters, `ls -l` format: type char (`-` file, `d` dir, `l` symlink,
`s` socket, `p` fifo, `b` block, `c` char) followed by three rwx triples.
Setuid/setgid/sticky replace the corresponding `x` with `s`/`s`/`t` (uppercase
when the execute bit is clear). Example: `drwxr-xr-x`, `-rwsr-xr-x`.

### 6.2 Human size
Binary units, base 1024: `B`, `KiB`, `MiB`, `GiB`, `TiB`. Zero decimals for
bytes, one decimal above. `0 B`, `999 B`, `1.0 KiB`, `1.4 KiB`, `1.0 MiB`.

---

## 7. Testing contract

Rust, `cargo test`:
- `util`: mode-string formatting across all type chars and the setuid/setgid/
  sticky matrix; human-size at every unit boundary and at zero.
- `path`: every rejection rule in §3.2, each asserting the specific `ErrorKind`.
- `ops`: copy and move against `tempfile::tempdir()` — files, nested
  directories, symlink preservation, collision without `overwrite`, collision
  with `overwrite`, `WouldRecurse` on self-nesting.
- `error`: `io::Error` → `ErrorKind` mapping for each row of the §2.4 table.
- `size`: recursive walk against `tempfile::tempdir()` — a known-byte tree sums
  exactly; a symlinked subdirectory is not descended into and not double-counted;
  a hard-linked file counts once; a `chmod 000` subdirectory yields
  `complete: false` with a non-zero `errorCount` and a correct partial sum; an
  empty directory yields zero; cancellation resolves as `Cancelled`.

Edge cases that must stay exercised somewhere in the suite: zero-byte files,
names at the 255-byte limit, broken symlinks, permission-denied directories,
a file removed mid-operation, and names containing spaces, newlines, and
non-ASCII characters.

Manual, per phase: the exit checks in `IMPLEMENTATION_PLAN.md`. A phase is not
done until its exit check passes on a real run, not just a compile.

---

## 8. Security posture

- The app runs strictly as the invoking user. No `sudo`, no polkit, no setuid
  helper, ever.
- Tauri config: `withGlobalTauri` disabled, CSP set to `default-src 'self'`,
  no remote URLs in the allowlist, shell/http/fs plugin scopes empty. The
  webview loads only bundled assets.
- No telemetry, no network access from either side of the app.
- Error messages shown to the user include paths but never file contents.
- `unsafe` does not appear in this codebase. If something appears to require
  it, stop and raise it rather than writing it.
- Run the security-audit skill against the backend before any build that could
  be reachable beyond localhost.
