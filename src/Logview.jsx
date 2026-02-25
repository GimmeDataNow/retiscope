import { createSignal, onMount, onCleanup, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function Logview() {
  const [logs, setLogs] = createSignal([]);
  const [isRunning, setIsRunning] = createSignal(false);
  const [userInput, setUserInput] = createSignal("--cli");
  let unlistenFunc;

  const startProcess = async () => {
    if (!userInput()) return;
    
    setLogs([]);
    try {
      // Set state BEFORE the await to prevent double-starts from rapid clicks
      setIsRunning(true); 
      await invoke("start_logging_process", { commandOptions: userInput() });
    } catch (e) {
      setLogs((p) => [...p, `Error: ${e}`]);
      setIsRunning(false);
    }
  };

  const stopProcess = async () => {
    try {
      await invoke("stop_logging_process");
    } catch (e) {
      console.error("Stop error:", e);
    } finally {
      // Always flip the UI state back, even if Rust says "No process was running"
      setIsRunning(false);
    }
  };

  onMount(async () => {
    // Correctly await the unlisten function
    const unlisten = await listen("process-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });
    unlistenFunc = unlisten;
  });

  onCleanup(() => {
    if (unlistenFunc) unlistenFunc();
    stopProcess();
  });

  const handleButtonClick = async (e) => {
    e.preventDefault();
    
    const currentlyRunning = isRunning();
    console.log("%c Button Clicked!", "color: yellow; font-weight: bold;");
    console.log("Current Signal State (isRunning):", currentlyRunning);
  
    if (currentlyRunning) {
      console.log("Action: Calling STOP");
      // Force the signal to false immediately to update UI
      setIsRunning(false); 
      try {
        await invoke("stop_logging_process");
      } catch (err) {
        console.error("Stop Command Failed:", err);
      }
    } else {
      console.log("Action: Calling START");
      setIsRunning(true);
      try {
        await invoke("start_logging_process", { commandOptions: userInput() });
      } catch (err) {
        console.error("Start Command Failed:", err);
        setIsRunning(false);
      }
    }
  };
  
  return (
    <div style={{ background: "#1e1e1e", color: "#d4d4d4", height: "100vh", display: "flex", "flex-direction": "column" }}>
      <div style={{ padding: "1rem", "border-bottom": "1px solid #444", display: "flex", gap: "10px" }}>
        <input 
          type="text" 
          disabled={isRunning()} // Prevent changing args while running
          value={userInput()} 
          onInput={(e) => setUserInput(e.target.value)}
          placeholder="Enter command"
          style={{ flex: 1, padding: "8px", background: isRunning() ? "#222" : "#333", color: "white", border: "1px solid #555" }}
        />
        
        <button 
          type="button"
          onClick={handleButtonClick}
          style={{ 
            background: isRunning() ? "#ff4d4d" : "#4CAF50", 
            color: "white", border: "none", padding: "8px 20px", cursor: "pointer",
            "font-weight": "bold"
          }}
        >
          {isRunning() ? "STOP PROCESS" : "RUN COMMAND"}
        </button>
      </div>

      <div style={{ flex: 1, "overflow-y": "auto", padding: "1rem", "font-family": "monospace", "white-space": "pre-wrap" }}>
        <For each={logs()}>
          {(log) => (
            <div style={{ color: log.startsWith("[ERR]") ? "#f44" : "#d4d4d4", "border-bottom": "1px solid #2a2a2a" }}>
              {log}
            </div>
          )}
        </For>
      </div>
    </div>
  );
}

export default Logview;
