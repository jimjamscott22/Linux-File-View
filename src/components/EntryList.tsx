import { useEffect, useRef } from "react";
import type { DirEntry } from "../types";
import { EntryRow } from "./EntryRow";

interface Props { entries: DirEntry[]; cursor: number; onSelect: (index: number) => void; onOpen: () => void }
export function EntryList({ entries, cursor, onSelect, onOpen }: Props) {
  const activeRef = useRef<HTMLDivElement>(null);
  useEffect(() => { activeRef.current?.scrollIntoView({ block: "nearest" }); }, [cursor]);
  return <section className="min-h-0 overflow-auto" role="table" aria-label="Directory entries">
    <div role="row" className="grid grid-cols-[minmax(16rem,1fr)_6rem_11rem] gap-4 px-5 py-2 border-b border-slate-700 text-slate-400 whitespace-nowrap">
      <span role="columnheader">NAME</span><span role="columnheader" className="text-right">SIZE</span><span role="columnheader" className="text-right">MODIFIED</span>
    </div>
    {entries.length === 0 && <p className="px-5 py-4 text-slate-500">No entries</p>}
    {entries.map((entry, index) => <div key={entry.path} ref={index === cursor ? activeRef : undefined}>
      <EntryRow entry={entry} active={index === cursor} onClick={() => onSelect(index)} onOpen={onOpen} />
    </div>)}
  </section>;
}
