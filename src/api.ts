import { invoke } from "@tauri-apps/api/core";
import type { DirEntry, ListOptions } from "./types";

export const resolveDir = (path: string) => invoke<string>("resolve_dir", { path });
export const listDir = (path: string, options: ListOptions) =>
  invoke<DirEntry[]>("list_dir", { path, options });
export const homeDir = () => invoke<string>("home_dir");
export const parentDir = (path: string) =>
  invoke<string | null>("parent_dir", { path });
