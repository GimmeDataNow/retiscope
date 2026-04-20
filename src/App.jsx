import { A } from "@solidjs/router";
import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import "./App.css";
import { Sidebar } from "./components/Sidebar.jsx";
import { onMount } from "solid-js";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  const [announces, setAnnounces] = createSignal([]);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  onMount(async () => {

    const unlistenAnnounce = await listen('announce_changed', (event) => {
      console.log('Announce update', event.payload);
    });
    
    const unlistenNodes = await listen('node_changed', (event) => {
      console.log('Node update:', event.payload);
    });

    try {
      const result = await invoke("fetch_announces_db");
      console.log("Fetched Announces:", result);
      setAnnounces(result);
    } catch (error) {
      console.error("Failed to fetch announces:", error);
    }

    try {
      const result = await invoke("fetch_nodes_db");
      console.log("Fetched Announces:", result);
      setAnnounces(result);
    } catch (error) {
      console.error("Failed to fetch announces:", error);
    }

    // clean up listeners when component unmounts
    // onCleanup(() => { unlistenAnnounce(); unlistenNodes(); });
  });

  return (
    <div class="container" style={{  }}>
      <h1>Retiscope</h1>
      The Reticulum network visualizer
      <A href="logview">hello</A>
    </div>
  );
}

export default App;
