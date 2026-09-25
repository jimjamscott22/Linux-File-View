import type { DirEntry } from "../types";

interface Props { entry: DirEntry; active: boolean; onClick: () => void; onOpen: () => void }
export function EntryRow({ entry, active, onClick, onOpen }: Props) {
  const isDirectory = entry.kind === "directory" || entry.symlinkTargetKind === "directory";
  const size = entry.kind === "directory" ? "—" : formatSize(entry.size);
  const modified = entry.modifiedMs === null ? "—" : new Date(entry.modifiedMs).toLocaleString("sv-SE", { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false });
  return <div role="row" aria-selected={active} onClick={onClick} onDoubleClick={onOpen}
    className={`grid grid-cols-[minmax(16rem,1fr)_6rem_11rem] gap-4 px-5 py-1 cursor-default whitespace-nowrap ${active ? "bg-emerald-400 text-slate-950" : "hover:bg-slate-800"}`}>
    <span role="cell" className="truncate">{isDirectory ? "▸ " : "  "}{entry.name}{entry.isSymlink ? " @" : ""}</span>
    <span role="cell" className="text-right">{size}</span>
    <span role="cell" className="text-right">{modified}</span>
  </div>;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KiB", "MiB", "GiB", "TiB"];
  let size = bytes / 1024;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) { size /= 1024; unit++; }
  return `${size.toFixed(1)} ${units[unit]}`;
}
