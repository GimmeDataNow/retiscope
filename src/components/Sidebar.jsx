// components/Sidebar.jsx
import { A } from "@solidjs/router";
import "./Sidebar.css";
import myIcon from "../assets/icon.svg";

export function Sidebar(props) {
  return (
    <div class="layout-wrapper">
      
      <nav class="menu">
        <div class="title-wrapper">
          <img src={myIcon} alt="My Icon" width="24" height="24" class ="icon"/>
          <h1 class="title">Retiscope</h1>
        </div>
        <A href="/" end>Dashboard</A>
        <A href="/graph">Node Graph</A>
        <A href="/pathtable">Path Table</A>
        <A href="/announcetable">Announce Table</A>
        <A href="/announcestream">Announce Stream</A>
        <A href="/connections">Connections</A>
        <A href="/logview">Logs</A>
      </nav>




      <main class="content-area" style={{ flex: 1 }}>
        {props.children} 
      </main>
    </div>
  );
}
