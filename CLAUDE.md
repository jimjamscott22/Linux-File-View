# CLAUDE.md — Terminal File Manager (`tfm`)

Guidance for Claude Code working in this repo. Read alongside `docs/SPEC.md` and
`docs/IMPLEMENTATION_PLAN.md`. When in doubt, the SPEC is the source of truth for the
contract; this file is the source of truth for *how* to build it.

## Project summary
A local-first Linux file manager. Rust backend (Tauri 2) owns all filesystem
operations and OS integration; React + TypeScript + Vite + Tailwind frontend
renders a dark, terminal-aesthetic UI inside the webview. Linux x86_64 only for v1.

## Stack
- **Backend:** Rust, Tauri 2.x
- **Frontend:** React + TypeScript + Vite + Tailwind
- **State:** Zustand (frontend store)
- **Key crates:** `notify`, `trash`, `opener`, `infer`, `serde`/`serde_json`, `thiserror`

## Commands
```bash
npm install                 # install frontend deps
npm run tauri dev           # run app in dev (hot reload)
npm run tauri build         # release bundle (.deb / AppImage)
npm run build               # frontend typecheck + build only
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test  --manifest-path src-tauri/Cargo.toml
```
Run `cargo clippy` and `cargo fmt` before considering any backend change done.
Run `npm run build` to catch TS errors before considering frontend work done.

## Architecture rules
- **The frontend never touches the filesystem.** Every FS interaction is a Tauri
  `#[command]`. No `fs` access from JS, no shelling out from the frontend.
- **Commands are thin.** Command functions validate/canonicalize input and delegate
  to module logic (`read`, `ops`, `open`, `watch`, `size`). Keep business logic
  out of `main.rs`.
- **One module per concern** — follow the layout in docs/IMPLEMENTATION_PLAN.md. Don't
  collapse `read`/`ops`/`watch`/`size` into one file.
- **Types are the contract.** `DirEntry`, `EntryInfo`, and `AppError` shapes must
  match docs/SPEC.md exactly; if you change one, update docs/SPEC.md in the same change.

## Rust conventions
- **No `unwrap()` / `expect()` / `panic!` in command paths or anything reachable
  from a command.** All IO returns `Result<T, AppError>`. `AppError` is a
  `thiserror` enum with distinct kinds (NotFound, PermissionDenied, CrossDevice,
  AlreadyExists, Io, …) serialized to `{ kind, message }` for the frontend.
- Map `std::io::Error` to the right `AppError` kind; never leak a raw error string
  as the only signal.
- **Canonicalize every path** coming from the frontend before acting on it. Define
  symlink behavior explicitly; do not follow symlinks implicitly during recursive
  ops.
- Prefer `std::fs` + `std::os::unix` for metadata; reach for `nix`/`users` only when
  std can't express it (owner/group names).
- Keep `unsafe` out of this codebase. If something seems to need it, stop and flag.

## Frontend conventions
- TypeScript strict mode on. No `any` — type the `invoke` wrappers in `api.ts` and
  reuse those types everywhere.
- All backend calls go through `api.ts`; components never call `invoke` directly.
- All event subscriptions go through `events.ts`.
- Keyboard handling is centralized in `hooks/useKeyboard.ts` → store actions. Don't
  scatter key listeners across components.
- Styling via Tailwind utility classes + the terminal tokens in `theme.css`. Keep
  the aesthetic consistent: monospace, dark background, single accent color, no
  rounded-corner/drop-shadow "web app" look.

## Safety & security (high priority)
This tool performs destructive filesystem operations and may eventually be reachable
on the home lab (the Pi has WireGuard exposure), so treat safety as a feature:
- **Delete routes to Trash** (`trash` crate), never `rm`/hard-delete in v1. A
  hard-delete path, if ever added, must be a separate, explicitly-labeled command
  with its own confirmation.
- Destructive ops (delete, overwrite-on-move) require a frontend confirmation and a
  backend that refuses to clobber silently — surface `AlreadyExists` and let the UI
  decide.
- Never run with elevated privileges; the app operates strictly as the invoking user.
- Validate that operation targets stay within expected bounds after canonicalization;
  reject obviously dangerous inputs rather than "trying anyway."
- When the security-audit skill is available, run it against the backend before any
  build that could be deployed beyond localhost.

## Testing expectations
- Unit-test `util` (permission-string and human-size formatting) and the copy/move
  logic against `tempfile::tempdir()`.
- Keep the edge-case list exercised: zero-byte files, very long names, broken
  symlinks, permission-denied dirs, files that vanish mid-operation, names with
  spaces/unicode.
- Each phase has an exit check in docs/IMPLEMENTATION_PLAN.md — don't mark a phase done
  until its exit check passes on a real run, not just a compile.

## Workflow notes
- Build in the phase order from docs/IMPLEMENTATION_PLAN.md; keep the app runnable at the
  end of every phase.
- When you change the command surface, update docs/SPEC.md's API table and the `api.ts`
  types together — they must not drift.
- Small, reviewable commits per logical unit (one command + its UI, one module).
- Don't add scope from the Non-Goals list (dual-pane, thumbnails, archives, network
  mounts) without it being asked for — note the idea and move on.
