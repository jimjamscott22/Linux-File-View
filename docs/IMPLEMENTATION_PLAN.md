# IMPLEMENTATION_PLAN.md — Terminal File Manager (`tfm`)

Build order, module layout, and per-phase exit checks. `SPEC.md` is the
contract; this is the route to it. `CLAUDE.md` is the house style.

**Rule for every phase: the app runs at the end of it.** No phase leaves the
tree in a state where `npm run tauri dev` fails to launch.

---

## Module layout

Create this shape in Phase 0 and keep it. Do not collapse `read` / `ops` /
`watch` into one file, and do not let logic drift into `main.rs`.

```
src-tauri/
├─ Cargo.toml
├─ tauri.conf.json
└─ src/
   ├─ main.rs        # tauri::Builder, invoke_handler registration, state. No logic.
   ├─ commands.rs    # every #[command]. Validate → delegate → return. Thin.
   ├─ error.rs       # AppError enum (thiserror), io::Error mapping, Serialize impl.
   ├─ model.rs       # DirEntry, EntryInfo, ListOptions, OpOutcome, Place, TextHead, DirSize.
   ├─ path.rs        # canonicalization + all validation rules (SPEC §3.2).
   ├─ read.rs        # list_dir, entry_info, read_text_head, sorting.
   ├─ size.rs        # recursive directory walk, cancellation flag, progress events.
   ├─ ops.rs         # create, rename, copy, move, trash. All mutation logic.
   ├─ open.rs        # open_path, reveal_in_terminal, terminal detection.
   ├─ watch.rs       # notify watcher, debounce, event emission, WatcherState.
   └─ util.rs        # mode_string, human_size, time conversion. Pure, tested.

src/
├─ main.tsx
├─ App.tsx           # layout shell only
├─ api.ts            # typed invoke wrappers — the ONLY place invoke is called
├─ events.ts         # the ONLY place listen() is called
├─ types.ts          # mirrors model.rs / error.rs exactly
├─ store.ts          # Zustand store + all actions
├─ theme.css         # terminal design tokens
├─ hooks/
│  ├─ useKeyboard.ts # every key binding, dispatching to store actions
│  ├─ useWatch.ts    # subscribe/unsubscribe on cwd change
│  └─ useDirSize.ts  # directory_size lifecycle: start, progress, cancel, cache
└─ components/
   ├─ PathBar.tsx
   ├─ Places.tsx
   ├─ EntryList.tsx
   ├─ EntryRow.tsx
   ├─ Details.tsx
   ├─ StatusBar.tsx
   ├─ Dialog.tsx     # generic modal: confirm / prompt
   └─ KeyHelp.tsx
```

Cargo dependencies, added as their phase arrives — not all up front:

| Crate | Phase | Use |
|---|---|---|
| `serde`, `serde_json` | 0 | wire types |
| `thiserror` | 0 | `AppError` |
| `tauri` 2.x | 0 | runtime |
| `users` | 2 | uid/gid → names |
| `infer` | 3 | content-sniffed MIME |
| `opener` | 3 | xdg-open |
| `trash` | 4 | freedesktop Trash |
| `notify` | 5 | filesystem watching |
| — | 6 | recursive sizing needs no new crate; `std::fs` + `std::os::unix` suffice |
| `tempfile` (dev) | 4 | `ops` tests |

---

## Phase 0 — Scaffold and contract skeleton

Get an empty but correctly-shaped app running.

1. `npm create tauri-app@latest` → React + TypeScript + Vite. Add Tailwind and
   Zustand. Strip the template's demo content.
2. Create every file in the layout above, empty or stubbed.
3. Implement `error.rs` fully — the `AppError` enum, the `io::Error` mapping
   table from SPEC §2.4, and its `Serialize` impl producing `{ kind, message }`.
4. Implement `model.rs` fully — all structs from SPEC §2, `rename_all =
   "camelCase"`, no logic.
5. Mirror them in `types.ts` by hand.
6. Implement `util.rs`: `mode_string`, `human_size`, `system_time_to_ms`. Write
   their unit tests now (SPEC §7) — these are the only things testable this
   early, and they anchor the formatting contract.
7. Lock down `tauri.conf.json` per SPEC §8: CSP `default-src 'self'`,
   `withGlobalTauri: false`, empty plugin scopes.
8. Wire one throwaway `ping` command end-to-end through `api.ts` to prove the
   bridge.

**Exit check:** `npm run tauri dev` opens a window that renders a value fetched
from a Rust command. `cargo test` passes with the `util` tests green.
`cargo clippy -- -D warnings` and `npm run build` are both clean.

---

## Phase 1 — Read-only browsing

The core loop: see a directory, move a cursor, navigate.

1. `path.rs`: canonicalization plus the validation rules from SPEC §3.2. Write
   the rejection tests alongside.
2. `read.rs`: `list_dir` — `read_dir` + `lstat` per entry, one `stat` for
   symlink targets, hidden filtering, and the sort ordering from SPEC §2.5.
3. `commands.rs`: `list_dir`, `home_dir`, `parent_dir`.
4. `store.ts`: `cwd`, `entries`, `cursor`, `listOptions`, `loading`, `error`,
   plus `navigate`, `moveCursor`, `refresh`.
5. `EntryList` / `EntryRow` / `PathBar` / `StatusBar`, styled from `theme.css`.
   Fixed-width `ls -l`-style columns, inverted-block cursor. The Size column
   renders `—` for directories, never the inode size (SPEC §5.5).
6. `useKeyboard.ts`: navigation, `.` hidden toggle, `s`/`S` sort, refresh.

**Exit check:** launch, land in `$HOME`, walk down several levels and back up
with `j`/`k`/`l`/`h` only. Toggle hidden files and each sort key and see the
order change. Navigate into a permission-denied directory and see a readable
`PermissionDenied` message in the status bar — not a blank list, not a crash.

---

## Phase 2 — Details pane

1. `read.rs`: `entry_info` — full `lstat`, `users` lookup for owner/group with
   numeric fallback, symlink resolution, non-recursive `childCount`.
2. `commands.rs`: `entry_info`.
3. `Details.tsx` rendering the full `EntryInfo`.
4. Debounced fetch (120 ms) on cursor change, per SPEC §5.4.

Directory size stays `—  (press u to measure)` in this phase — the recursive
walk is Phase 6. Nothing here may walk a subtree.

**Exit check:** cursor through a directory holding `j` — the pane keeps up and
does not fire a command per row (verify by log or counter). Check a regular
file, a directory, a symlink, a broken symlink, and something under `/dev`:
each renders correct metadata with no panic and no `unwrap`. Cursor over a
directory containing a very large subtree and confirm the pane responds
instantly — proof that nothing is walking it.

---

## Phase 3 — Open and preview

1. `open.rs`: `open_path` via `opener`; `reveal_in_terminal` with the terminal
   detection chain from SPEC §4.4.
2. `read.rs`: `read_text_head` with binary detection (NUL sample).
3. `infer` for `mimeType` in `entry_info`.
4. Wire `Enter`/`l` on a file → open; `o` → open; `t` → terminal.
5. Text head shown in the details pane for text-ish files.

**Exit check:** `Enter` on a `.txt`, an image, and a `.pdf` each launch the
right default application. `t` opens a terminal in the current directory. A
binary file shows `isBinary` rather than mojibake. A file with no handler
surfaces `Unsupported` rather than hanging.

---

## Phase 4 — Mutations

The dangerous phase. Nothing here ships without its confirmation UI.

1. `ops.rs`, in this order, each with `tempfile` tests before the UI:
   `create_dir`, `create_file`, `rename_entry`, `copy_entries`, `move_entries`,
   `trash_entries`.
2. Copy: recursive, symlink-preserving, permission-bit copying, collision →
   `AlreadyExists` unless `overwrite` (SPEC §4.2).
3. Move: `fs::rename` fast path, `EXDEV` → copy-then-trash-source (SPEC §4.3).
4. Trash via the `trash` crate. **No hard-delete command exists.**
5. `WouldRecurse` guard on both copy and move.
6. `OpOutcome` partial-success reporting — batches never abort on one failure.
7. `Dialog.tsx`: confirm and prompt modals. Bind `n`, `N`, `r`, `y`, `x`, `p`,
   `d`. Every destructive path routes through a confirmation naming the
   entries; overwrite is a second, separate confirmation.

**Exit check:** in a scratch directory — create, rename, copy a nested tree,
move within a filesystem, move across filesystems (`/tmp` → `$HOME` if they
differ), and trash a selection; verify the trashed items appear in the desktop
Trash. Attempt to copy a directory into itself and see `WouldRecurse`. Attempt
an overwrite and confirm nothing is clobbered until the second confirmation.
Trash a multi-select where one entry is unwritable and confirm the rest still
succeed, with the failure listed.

---

## Phase 5 — Live watching

1. `watch.rs`: single non-recursive `notify` watcher held in Tauri state,
   150 ms debounce, `fs://changed` emission, `fs://watch-error` on failure.
2. `commands.rs`: `watch_dir`, `unwatch_all`. Re-watching replaces the
   existing watcher.
3. `events.ts` + `useWatch.ts`: subscribe on `cwd` change, refresh on event.
4. Selection and cursor reconciliation by path, per SPEC §5.4.

**Exit check:** with the app open on a directory, `touch`, `mv`, and `rm` files
in it from an external shell — the list updates within ~200 ms. Create 200
files in a loop and confirm the debounce coalesces them rather than issuing 200
reloads. Multi-select several entries, cause an external change, and confirm
the selection survives by path. `rm -r` the watched directory and confirm a
clean `fs://watch-error` rather than a hang.

---

## Phase 6 — Recursive directory sizes

On-demand only, one directory at a time. Sequenced here because it reuses the
event plumbing from Phase 5 and the cache invalidation depends on the watcher.

1. `size.rs`: depth-first walk over `read_dir` with `lstat` only. Never follows
   symlinks. Skips `(dev, inode)` pairs already seen so hard links count once.
   Sums apparent size. Unreadable subtree → skip, `errorCount += 1`,
   `complete = false`, walk continues.
2. Cancellation: an `AtomicBool` per in-flight measurement held in Tauri state,
   checked between entries. `cancel_directory_size` sets it; the walk returns
   `Cancelled`. Starting a new measurement cancels the previous — at most one
   runs at a time.
3. Progress: `dirsize://progress` emitted at most every 100 ms, throttled in
   `size.rs` rather than per-entry.
4. `commands.rs`: `directory_size` (async), `cancel_directory_size`.
5. Tests before the UI (SPEC §7): exact sum on a known-byte tree, symlinked
   subdirectory not descended, hard link counted once, `chmod 000` subdir →
   partial with correct `errorCount`, empty directory → zero, cancellation →
   `Cancelled`.
6. `useDirSize.ts` + the `dirSizes` store slice: start on `u`, render the
   running total live, cancel on `Esc`, cache by canonical path, invalidate on
   `fs://changed` / refresh / navigation (SPEC §4.6).
7. Details pane rendering: `—  (press u to measure)` → running total →
   final value labelled `apparent`, with `partial — N unreadable` when
   incomplete and a timestamp once older than 60 s.

`Cancelled` is an ordinary outcome, not a status-bar error.

**Exit check:** press `u` on a large tree (`~/.cache` or a `node_modules`) —
the total climbs live and the UI stays responsive throughout. `Esc` mid-walk
stops it promptly with no error shown. Measure a directory containing a
`chmod 000` subdirectory and see a partial result, not a failure. Measure a
directory containing a symlink loop and confirm it terminates. Compare a
clean tree's result against `du -sb --apparent-size` and get the same byte
count. Move the cursor to another directory mid-walk and confirm the first
measurement is cancelled rather than left running.

---

## Phase 7 — Filter, places, help

1. `places` command: Home plus the XDG user dirs that actually exist.
2. `Places.tsx` sidebar.
3. Frontend-only substring filter on `/` (SPEC §5.3), `Esc` to clear.
4. `KeyHelp.tsx` overlay on `?`, generated from the same binding table
   `useKeyboard.ts` uses — one source, so it cannot drift.

**Exit check:** `/` filters live as you type and never issues a `list_dir`
(verify by log). Clicking each place navigates there. `?` lists every binding
that actually works, with no entry for one that does not.

---

## Phase 8 — Hardening and packaging

1. Sweep for `unwrap` / `expect` / `panic!` anywhere reachable from a command:
   `rg '\.(unwrap|expect)\(|panic!' src-tauri/src`. The only acceptable hits
   are inside `#[cfg(test)]`.
2. Run the full SPEC §7 edge-case list by hand against a purpose-built fixture
   directory: zero-byte file, 255-byte name, broken symlink, `chmod 000`
   directory, unicode and newline names, a file deleted mid-copy, a symlink
   loop and a hard-linked file under `u`.
3. Run the security-audit skill against the backend. Confirm the SPEC §8
   posture in `tauri.conf.json` line by line.
4. `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `npm run build` —
   all clean.
5. `npm run tauri build` → `.deb` and AppImage. Install the `.deb` on a clean
   machine or container and launch it.
6. README: screenshot, keybinding table, install instructions.

**Exit check:** the installed `.deb` launches and completes a full session —
browse, inspect, open, copy, move, trash, watch — with no console errors and
no unhandled command rejections.

---

## Commit discipline

One logical unit per commit: one command plus the UI that calls it, or one
module. A command and its `api.ts` wrapper and its `types.ts` type land
together. A change to any wire type lands with its `SPEC.md` edit in the same
commit — the API table and `api.ts` must never drift.
