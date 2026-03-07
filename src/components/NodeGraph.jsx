import { onMount, createEffect, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Graph from "graphology";
import Sigma from "sigma";
import forceAtlas2 from "graphology-layout-forceatlas2";

export default function NodeGraph() {
  let container;
  const [data] = createResource(async () => await invoke("get_graph_data"));

  const stringifyId = (val) => {
    if (!val) return "unknown";
    
    // 1. If it's already a simple string (e.g., "cbbd5f99...")
    if (typeof val === 'string') return val;
  
    // 2. If it's the SurrealDB RecordID object
    if (typeof val === 'object') {
      // Check for the new SurrealDB 3.0 structure: { table: "...", id: { String: "..." } }
      // Note: Sometimes SurrealDB uses 'id' and sometimes 'key' depending on the driver version
      const innerId = val.id || val.key; 
  
      if (innerId && typeof innerId === 'object') {
        // It's the { String: "hex" } or { Number: 123 } variant
        return innerId.String || innerId.Number || JSON.stringify(innerId);
      }
      
      // Check for the { tb: "...", id: "..." } variant
      if (val.tb && val.id) {
        return typeof val.id === 'object' ? (val.id.String || JSON.stringify(val.id)) : val.id;
      }
  
      // Last resort fallback for objects
      return JSON.stringify(val);
    }
  
    return String(val);
  };

  createEffect(() => {
    const rawData = data();
    if (!rawData || !container) return;

    // --- DEBUGGING ---
    // console.log("First Entry Raw Data:", rawData[0]);
    // console.log("Sample Stringified ID:", stringifyId(rawData[0]?.id));
    // -----------------

    const graph = new Graph();

    graph.addNode("origin", { 
      label: "Origin", size: 15, color: "#ff4757", x: 0, y: 0 
    });

    // PASS 1: Create all nodes
    rawData.forEach((entry) => {
      const nodeId = stringifyId(entry.id);
      const ifaceId = stringifyId(entry.iface);
      const recvId = entry.received_from ? stringifyId(entry.received_from) : null;

      if (!graph.hasNode(ifaceId)) {
        graph.addNode(ifaceId, { 
          label: `IF: ${ifaceId}`, 
          size: 10, color: "#2ed573" 
        });
        // Connect to origin immediately
        graph.addEdge("origin", ifaceId, { color: "#2ed573", weight: 2 });
      }

      if (!graph.hasNode(nodeId)) {
        graph.addNode(nodeId, { 
          label: nodeId, 
          size: 6, color: "#1e90ff" 
        });
      }

      if (recvId && !graph.hasNode(recvId)) {
        graph.addNode(recvId, { 
          label: `R: ${recvId}`, 
          size: 6, color: "#747d8c" 
        });
      }
    });

    // --- PASS 2: Add ALL Edges with Chaining Logic ---
    rawData.forEach((entry) => {
      const nodeId = stringifyId(entry.id);
      const ifaceId = stringifyId(entry.iface);
      const recvId = entry.received_from ? stringifyId(entry.received_from) : null;
    
      // 1. Every interface must connect to Origin (The Anchor)
      if (!graph.hasEdge("origin", ifaceId)) {
        graph.addEdge("origin", ifaceId, { color: "#2ed573", size: 3 });
      }
    
      // 2. Routing Logic
      if (entry.hops === 0) {
        // This node is physically seen by the interface
        if (!graph.hasEdge(nodeId, ifaceId)) {
          graph.addEdge(nodeId, ifaceId, { color: "#555", type: "line" });
        }
      } else if (recvId) {
        // This node is seen via a relay. 
        // Connect the node to its reporter (the gray node)
        if (graph.hasNode(nodeId) && graph.hasNode(recvId)) {
          if (!graph.hasEdge(nodeId, recvId)) {
            graph.addEdge(nodeId, recvId, { color: "#777", type: "line" });
          }
        }
        
        // CRITICAL: We also need to make sure the Relay (recvId) eventually 
        // connects to the interface (ifaceId) so the cluster isn't floating.
        if (graph.hasNode(recvId) && graph.hasNode(ifaceId)) {
          if (!graph.hasEdge(recvId, ifaceId) && recvId !== ifaceId) {
            // We link the relay to the interface it's reporting through
            graph.addEdge(recvId, ifaceId, { color: "#ffa502", weight: 0.5 });
          }
        }
      }
    });

    // Layout
    graph.nodes().forEach(n => {
      if(n === "origin") return;
      graph.setNodeAttribute(n, "x", Math.random());
      graph.setNodeAttribute(n, "y", Math.random());
    });

    forceAtlas2.assign(graph, { 
        iterations: 150, 
        settings: { 
            gravity: 0.8, 
            scalingRatio: 20, 
            strongGravityMode: true 
        } 
    });

    const renderer = new Sigma(graph, container, {
        labelSize: 10,
        labelWeight: "bold",
        labelColor: { color: "#fff" }
    });

    onMount(() => () => renderer.kill());
  });

  return (
    <div style={{ width: "100%", height: "100vh", background: "#1a1a1a" }}>
      <div ref={container} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
