import { A } from "@solidjs/router";
import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Sidebar } from "./components/Sidebar.jsx";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  return (
    <div class="container" style={{  }}>
      <h1>Retiscope</h1>
      The Reticulum network visualizer
      <A href="logview">hello</A>
    </div>
  );
}

export default App;
