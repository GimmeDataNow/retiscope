import { createSignal, onMount, onCleanup, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function Logview() {
  const [logs, setLogs] = createSignal([]);
  const [isRunning, setIsRunning] = createSignal(false);
  let unlistenFunc;

  const startProcess = async () => {
    setLogs([]); // Clear old logs
    setIsRunning(true);
    await invoke("start_logging_process");
  };

  const stopProcess = async () => {
    try {
      await invoke("stop_logging_process");
      setIsRunning(false);
      setLogs((prev) => [...prev, "--- PROCESS TERMINATED BY USER ---"]);
    } catch (e) {
      console.error(e);
    }
  };

  onMount(async () => {
    unlistenFunc = await listen("process-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });
    // Auto-start on mount
    startProcess();
  });

  onCleanup(() => {
    if (unlistenFunc) unlistenFunc();
    stopProcess(); // Optional: Kill process when user leaves the page
  });

  return (
    <div style={{ background: "#1e1e1e", color: "#d4d4d4", height: "100vh", display: "flex", "flex-direction": "column" }}>
      <div style={{ padding: "1rem", "border-bottom": "1px solid #444", display: "flex", gap: "10px", "align-items": "center" }}>
        <h3 style={{ margin: 0 }}>Process Logs</h3>
        
        {isRunning() ? (
          <button onClick={stopProcess} style={{ background: "#ff4d4d", color: "white", border: "none", padding: "5px 15px", cursor: "pointer" }}>
            Stop Process
          </button>
        ) : (
          <button onClick={startProcess} style={{ background: "#4CAF50", color: "white", border: "none", padding: "5px 15px", cursor: "pointer" }}>
            Restart Process
          </button>
        )}
      </div>

      <div style={{ flex: 1, "overflow-y": "auto", padding: "1rem", "font-family": "monospace" }}>
        <For each={logs()}>
          {(log) => <div style={{ "margin-bottom": "2px" }}>{log}</div>}
        </For>
      </div>
    </div>
  );
}

export default Logview;
