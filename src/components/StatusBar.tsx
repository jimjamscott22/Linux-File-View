import type { AppError, ListOptions } from "../types";

interface Props { count: number; cursor: number; options: ListOptions; error: AppError | null }
export function StatusBar({ count, cursor, options, error }: Props) {
  return <footer className="border-t border-slate-700 px-5 py-2 text-xs text-slate-400 flex justify-between gap-4">
    <span>{error ? <span className="text-red-400">{error.kind}: {error.message}</span> : `${count} entries · ${count ? cursor + 1 : 0}/${count}`}</span>
    <span>{options.sort}{options.descending ? " ↓" : " ↑"} · {options.showHidden ? "hidden shown" : "hidden off"} · j/k move · h/l open · . hidden · s/S sort</span>
  </footer>;
}
