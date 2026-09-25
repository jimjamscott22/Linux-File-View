Paste this into the new session:

> Continue work on the Linux-File-View (`tfm`) project in `/home/jimjamscozz/Desktop/GitHub-Repos/Linux-File-View`. Follow `docs/SPEC.md` and `docs/IMPLEMENTATION_PLAN.md` in phase order, stopping between phases.
>
> Phase 0 was committed as `057bfc6`. The Phase 1 read-only browsing implementation was committed as `8cb3ab2`; the working tree was clean. Its build, Clippy, formatting check, and 11 Rust tests passed. The live app displayed the Home listing, and keyboard input moved between directories.
>
> **First finish Phase 1 verification:** confirm the hidden-file and sorting shortcuts and the permission-error status bar in a live desktop run. Automated keystrokes were unreliable in the previous session, so those checks remain unconfirmed. Do not call Phase 1 fully verified until they pass. Then report the result and stop before Phase 2.
>
> The editor’s Snap environment can break WebKit with a `libpthread.so.0` error. Launch Tauri with a clean host environment if needed. Read the current repo state before changing anything, preserve the existing commits, and run the relevant verification commands before making completion claims.
