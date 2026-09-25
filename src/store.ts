import { create } from "zustand";
import { homeDir, listDir, parentDir, resolveDir } from "./api";
import type { AppError, AppState, ListOptions, SortKey } from "./types";

interface Actions {
  initialize: () => Promise<void>;
  navigate: (path: string) => Promise<void>;
  refresh: () => Promise<void>;
  goParent: () => Promise<void>;
  goHome: () => Promise<void>;
  enter: () => Promise<void>;
  moveCursor: (delta: number) => void;
  selectIndex: (index: number) => void;
  selectFirst: () => void;
  selectLast: () => void;
  toggleHidden: () => Promise<void>;
  cycleSort: () => Promise<void>;
  toggleSortDirection: () => Promise<void>;
}

type FileStore = AppState & Actions;
const initialOptions: ListOptions = { sort: "name", descending: false, showHidden: false };
const sortKeys: SortKey[] = ["name", "size", "modified", "kind"];
let requestId = 0;

function asAppError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "kind" in error && "message" in error) {
    return error as AppError;
  }
  return { kind: "Io", message: String(error) };
}

export const useFileStore = create<FileStore>((set, get) => ({
  cwd: "",
  entries: [],
  listOptions: initialOptions,
  cursor: 0,
  selected: new Set<string>(),
  clipboard: null,
  filter: "",
  info: null,
  dirSizes: new Map(),
  dialog: null,
  error: null,
  loading: false,

  initialize: async () => {
    try {
      await get().navigate(await homeDir());
    } catch (error) {
      set({ error: asAppError(error), loading: false });
    }
  },
  navigate: async (path) => {
    const current = ++requestId;
    set({ loading: true, error: null });
    try {
      const cwd = await resolveDir(path);
      const entries = await listDir(cwd, get().listOptions);
      if (current === requestId) {
        set({ cwd, entries, cursor: 0, selected: new Set(), loading: false });
      }
    } catch (error) {
      if (current === requestId) set({ error: asAppError(error), loading: false });
    }
  },
  refresh: async () => {
    const { cwd, entries: previous, cursor, listOptions } = get();
    if (!cwd) return;
    const current = ++requestId;
    const cursorPath = previous[cursor]?.path;
    set({ loading: true, error: null });
    try {
      const entries = await listDir(cwd, listOptions);
      if (current === requestId) {
        const retained = entries.findIndex((entry) => entry.path === cursorPath);
        set({
          entries,
          cursor: retained >= 0 ? retained : Math.min(cursor, Math.max(0, entries.length - 1)),
          selected: new Set([...get().selected].filter((path) => entries.some((entry) => entry.path === path))),
          loading: false,
        });
      }
    } catch (error) {
      if (current === requestId) set({ error: asAppError(error), loading: false });
    }
  },
  goParent: async () => {
    const { cwd } = get();
    if (!cwd) return;
    try {
      const parent = await parentDir(cwd);
      if (parent) await get().navigate(parent);
    } catch (error) {
      set({ error: asAppError(error) });
    }
  },
  goHome: async () => {
    try {
      await get().navigate(await homeDir());
    } catch (error) {
      set({ error: asAppError(error) });
    }
  },
  enter: async () => {
    const { entries, cursor } = get();
    const entry = entries[cursor];
    if (entry?.kind === "directory" || entry?.symlinkTargetKind === "directory") {
      await get().navigate(entry.path);
    }
  },
  moveCursor: (delta) => set((state) => ({
    cursor: Math.max(0, Math.min(state.entries.length - 1, state.cursor + delta)),
  })),
  selectIndex: (index) => set((state) => ({ cursor: Math.max(0, Math.min(index, state.entries.length - 1)) })),
  selectFirst: () => set({ cursor: 0 }),
  selectLast: () => set((state) => ({ cursor: Math.max(0, state.entries.length - 1) })),
  toggleHidden: async () => {
    set((state) => ({ listOptions: { ...state.listOptions, showHidden: !state.listOptions.showHidden } }));
    await get().refresh();
  },
  cycleSort: async () => {
    set((state) => ({ listOptions: { ...state.listOptions, sort: sortKeys[(sortKeys.indexOf(state.listOptions.sort) + 1) % sortKeys.length] } }));
    await get().refresh();
  },
  toggleSortDirection: async () => {
    set((state) => ({ listOptions: { ...state.listOptions, descending: !state.listOptions.descending } }));
    await get().refresh();
  },
}));
