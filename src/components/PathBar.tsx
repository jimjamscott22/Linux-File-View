interface Props { path: string; loading: boolean }
export function PathBar({ path, loading }: Props) {
  return <header className="border-b border-slate-700 px-5 py-3 flex justify-between gap-4">
    <span className="text-emerald-400">tfm <span className="text-slate-200">{path || "loading home…"}</span></span>
    {loading && <span className="text-slate-400">loading…</span>}
  </header>;
}
