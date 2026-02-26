// components/Sidebar.jsx
import { A } from "@solidjs/router";

export function Sidebar(props) {
  return (
    <div class="layout-wrapper" style={{ display: "flex", "min-height": "100vh" }}>
      <nav class="sidebar" style={{ width: "200px", borderRight: "1px solid #ccc" }}>
        <ul>
          <li><A href="/">Dashboard</A></li>
          <li><A href="/logview">Logs</A></li>
        </ul>
      </nav>

      <main class="content-area" style={{ flex: 1 }}>
        {props.children} 
      </main>
    </div>
  );
}
