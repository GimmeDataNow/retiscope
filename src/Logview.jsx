import { createSignal, onMount, onCleanup, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function Logview() {
  const [logs, setLogs] = createSignal([]);
  const [isRunning, setIsRunning] = createSignal(false);
  const [userInput, setUserInput] = createSignal("--cli");
  
  let logContainerRef; // Reference for auto-scrolling
  const MAX_LOGS = 500; // Memory limit

  // AUTO-SCROLL LOGIC
  createEffect(() => {
    logs(); // Track the logs signal
    if (logContainerRef) {
      logContainerRef.scrollTop = logContainerRef.scrollHeight;
    }
  });

  onMount(async () => {
    const unlisten = await listen("process-log", (event) => {
      setLogs((prev) => {
        const newLogs = [...prev, event.payload];
        // Keep only the last MAX_LOGS entries
        return newLogs.slice(-MAX_LOGS);
      });
    });

    onCleanup(() => {
      unlisten();
    });
  });

  const handleButtonClick = async (e) => {
    e.preventDefault();
    if (isRunning()) {
      setIsRunning(false);
      await invoke("stop_logging_process").catch(console.error);
    } else {
      setLogs([]); // Clear logs on new run
      setIsRunning(true);
      try {
        await invoke("start_logging_process", { commandOptions: userInput() });
      } catch (e) {
        setIsRunning(false);
        setLogs([`Error: ${e}`]);
      }
    }
  };

  return (
    <div style={{ background: "#1e1e1e", color: "#d4d4d4", height: "100vh", display: "flex", "flex-direction": "column", flex: 1 }}>
      {/* Header / Controls */}
      <div style={{ padding: "1rem", "border-bottom": "1px solid #444", display: "flex", gap: "10px" }}>
        <input 
          type="text" 
          value={userInput()} 
          onInput={(e) => setUserInput(e.target.value)}
          style={{ flex: 1, padding: "8px", background: "#333", color: "white", border: "1px solid #555" }}
        />
        <button 
          onClick={handleButtonClick}
          style={{ 
            background: isRunning() ? "#ff4d4d" : "#4CAF50", 
            color: "white", border: "none", padding: "8px 20px", cursor: "pointer" 
          }}
        >
          {isRunning() ? "Stop" : "Run"}
        </button>
      </div>

      {/* Log Container */}
      <div 
        ref={logContainerRef} // Link the ref here
        style={{ 
          flex: 1, 
          "overflow-y": "auto", 
          padding: "1rem", 
          "font-family": "'Cascadia Code', 'Fira Code', monospace", 
          "white-space": "pre-wrap",
          "scroll-behavior": "smooth" // Optional: makes scrolling look nice
        }}
      >
        <For each={logs()}>
          {(log) => (
            <div style={{ 
              "border-left": "2px solid #333", 
              "padding-left": "8px", 
              "margin-bottom": "2px",
              "font-size": "13px"
            }}>
              {log}
            </div>
          )}
        </For>
      </div>
    </div>
  );
}

export default Logview;
