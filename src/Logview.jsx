import { createSignal, onMount, onCleanup, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function Logview() {
  const [logs, setLogs] = createSignal([]);
  let unlistenFunc; // Store the function directly

  onMount(async () => {
    // 1. Setup Listener
    // We assign the resolved promise to our variable
    unlistenFunc = await listen("process-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });

    // 2. Start the process
    try {
      await invoke("start_logging_process");
    } catch (e) {
      console.error("Failed to start process:", e);
    }
  });

  onCleanup(() => {
    // 3. Clean up the listener to prevent memory leaks
    if (unlistenFunc) {
      unlistenFunc();
    }
  });

  return (
    <div style={{ 
      background: "#1e1e1e", 
      color: "#d4d4d4", 
      padding: "1rem", 
      height: "100vh", 
      "font-family": "monospace",
      "overflow-y": "auto" 
    }}>
      <h3>Process Logs:</h3>
      <div class="log-container">
        <For each={logs()}>
          {(log) => <div style={{ "border-bottom": "1px solid #333" }}>{log}</div>}
        </For>
      </div>
    </div>
  );
}

export default Logview;
