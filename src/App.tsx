import { useState, useEffect } from "react";
import { ping } from "./api";

function App() {
  const [greetMsg, setGreetMsg] = useState("");

  useEffect(() => {
    ping().then(setGreetMsg).catch(console.error);
  }, []);

  return (
    <main className="min-h-screen bg-gray-900 text-gray-100 flex items-center justify-center font-mono">
      <div className="text-center">
        <h1 className="text-2xl font-bold mb-4">Terminal File Manager</h1>
        <p>Ping from Rust: <span className="text-green-400 font-bold">{greetMsg}</span></p>
      </div>
    </main>
  );
}

export default App;
