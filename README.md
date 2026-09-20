# Linux-File-View (`tfm`)

A local-first, keyboard-centric Linux file manager. A **Rust (Tauri v2)** backend owns all filesystem operations and OS integration safely and directly, while a **React + TypeScript + Vite + Tailwind** frontend renders a fast, dark, terminal-aesthetic UI inside the webview.

Built specifically for Linux x86_64, `tfm` provides lightning-fast keyboard-first navigation with the safety and capabilities of a modern file manager.

## Features

- **Keyboard-driven:** Navigate, browse, and perform file operations entirely via the keyboard without taking your hands off the home row.
- **Terminal aesthetic:** Monospace layout with fixed-width `ls -l` styling, custom mode string rendering, and no rounded "web app" distractions.
- **Accurate Linux metadata:** Inspect an entry's real metadata (permission bits, uid/gid lookup, times, exact sizes) instantly with a responsive details pane.
- **Safe destructive operations:** "Delete" is securely routed to the freedesktop Trash via the `trash` crate — no hard deletes exist in the codebase. All overwrite mutations and deletions enforce explicit confirmation dialogs.
- **Live filesystem watching:** Automatically reflects external file changes via `notify` with debounced, non-recursive polling.
- **Secure architecture:** The frontend never touches the filesystem; it operates exclusively as a view over the backend state. All mutations are Tauri `#[command]`s, and the app runs strictly under the invoking user.

## Keybindings

`tfm` takes heavy inspiration from `vim` and terminal standards for its keyboard mappings.

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
| `/` | Focus the filter input (frontend-only substring match) |
| `s` | Cycle sort key (name, size, modified, kind) |
| `S` | Toggle sort direction |
| `r` | Rename selected |
| `n` | New file |
| `N` | New directory |
| `y` | Yank (mark for copy) |
| `x` | Cut (mark for move) |
| `p` | Paste into current directory |
| `d` | Move to Trash (opens confirmation) |
| `u` | Compute recursive size of the cursor directory (on-demand) |
| `o` | Open with default application (`xdg-open`) |
| `t` | Open terminal here |
| `F5` / `Ctrl-r` | Refresh list |
| `?` | Toggle key help overlay |

## Development Setup

### Prerequisites

You will need standard Tauri dependencies for Linux development:
- `Node.js` (LTS) & `npm`
- `Rust` & `cargo`

```bash
# Install frontend dependencies
npm install

# Run the app in development mode (hot reloading)
npm run tauri dev
```

### Build & Packaging

```bash
# Frontend typecheck + build only
npm run build

# Build release bundle (.deb / AppImage)
npm run tauri build
```

## Architecture

- **Backend:** Rust, Tauri 2.x
- **Frontend:** React, TypeScript, Vite, Tailwind CSS
- **State Management:** Zustand
- **Key Rust Crates:** `notify`, `trash`, `opener`, `infer`, `serde`, `thiserror`

The frontend holds zero filesystem capability on its own. Every file operation involves canonicalized backend checks, and partial successes in batch operations (like `copy` or `move`) do not fail the entire batch.
