export type EntryKind = "file" | "directory" | "symlink" | "other";

export interface DirEntry {
  name: string;
  path: string;
  kind: EntryKind;
  size: number;
  modifiedMs: number | null;
  isHidden: boolean;
  isSymlink: boolean;
  symlinkTargetKind: EntryKind | null;
  isBrokenSymlink: boolean;
  readable: boolean;
}

export interface EntryInfo {
  name: string;
  path: string;
  kind: EntryKind;
  size: number;
  sizeHuman: string;
  modifiedMs: number | null;
  accessedMs: number | null;
  createdMs: number | null;
  mode: number;
  modeString: string;
  owner: string;
  group: string;
  uid: number;
  gid: number;
  nlink: number;
  inode: number;
  isHidden: boolean;
  isSymlink: boolean;
  symlinkTarget: string | null;
  symlinkResolved: string | null;
  isBrokenSymlink: boolean;
  mimeType: string | null;
  childCount: number | null;
}

export type ErrorKind =
  | "NotFound"
  | "PermissionDenied"
  | "AlreadyExists"
  | "NotADirectory"
  | "IsADirectory"
  | "CrossDevice"
  | "InvalidPath"
  | "DirectoryNotEmpty"
  | "WouldRecurse"
  | "Cancelled"
  | "Unsupported"
  | "Io";

export interface AppError {
  kind: ErrorKind;
  message: string;
}

export type SortKey = "name" | "size" | "modified" | "kind";

export interface ListOptions {
  sort: SortKey;
  descending: boolean;
  showHidden: boolean;
}

export interface DirSize {
  path: string;
  bytes: number;
  sizeHuman: string;
  fileCount: number;
  dirCount: number;
  complete: boolean;
  errorCount: number;
  measuredAtMs: number;
}

export interface TextHead {
  content: string;
  truncated: boolean;
  isBinary: boolean;
}

export interface Place {
  label: string;
  path: string;
}

export interface OpOutcome {
  succeeded: string[];
  failed: OpFailure[];
}

export interface OpFailure {
  path: string;
  error: AppError;
}

// Frontend specific types

export interface DirSizeStateRunning {
  status: "running";
  bytes: number;
  fileCount: number;
}

export interface DirSizeStateDone {
  status: "done";
  value: DirSize;
}

export type DirSizeState = DirSizeStateRunning | DirSizeStateDone;

export interface DialogState {
  type: "confirm" | "prompt";
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export interface AppState {
  cwd: string;
  entries: DirEntry[];
  listOptions: ListOptions;
  cursor: number;
  selected: Set<string>;
  clipboard: { mode: "copy" | "cut"; paths: string[] } | null;
  filter: string;
  info: EntryInfo | null;
  dirSizes: Map<string, DirSizeState>;
  dialog: DialogState | null;
  error: AppError | null;
  loading: boolean;
}
