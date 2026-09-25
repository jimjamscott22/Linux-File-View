import { useEffect } from "react";
import { EntryList } from "./components/EntryList";
import { PathBar } from "./components/PathBar";
import { StatusBar } from "./components/StatusBar";
import { useKeyboard } from "./hooks/useKeyboard";
import { useFileStore } from "./store";

function App() {
  const { cwd, entries, cursor, listOptions, loading, error, initialize, enter, selectIndex } = useFileStore();
  useKeyboard();
  useEffect(() => { void initialize(); }, [initialize]);
  return <main className="h-screen w-screen bg-slate-900 text-slate-100 font-mono flex flex-col overflow-hidden">
    <PathBar path={cwd} loading={loading} />
    <EntryList entries={entries} cursor={cursor} onSelect={selectIndex} onOpen={() => { void enter(); }} />
    <StatusBar count={entries.length} cursor={cursor} options={listOptions} error={error} />
  </main>;
}

export default App;
