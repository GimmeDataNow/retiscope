import { onMount, createEffect, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Graph from "graphology";
import Sigma from "sigma";
import forceAtlas2 from "graphology-layout-forceatlas2";

export default function NodeGraph() {
  let container;
  const [data] = createResource(async () => await invoke("get_graph_data"));

  const stringifyId = (val) => {
    if (!val) return null;
    if (typeof val === 'string') return val;
    if (typeof val === 'object') {
      // Prioritize the String key inside the id/key object
      const inner = val.id || val.key || val;
      if (inner && inner.String) return inner.String;
      if (inner && inner.Number) return String(inner.Number);
      if (val.tb && val.id) return typeof val.id === 'object' ? (val.id.String || JSON.stringify(val.id)) : val.id;
      return JSON.stringify(val);
    }
    return String(val);
  };

  createEffect(() => {
    const rawData = data();
    if (!rawData || !container) return;

    const graph = new Graph();
    
    // Identify who is a relay (anyone who appears in a 'received_from' field)
    const relays = new Set(rawData.map(d => stringifyId(d.received_from)).filter(Boolean));

    graph.addNode("origin", { 
      label: "Origin", size: 15, color: "#ff4757", x: 0, y: 0 
    });

    // PASS 1: Create Nodes with consistent coloring
    rawData.forEach((entry) => {
      const nodeId = stringifyId(entry.id);
      const ifaceId = stringifyId(entry.iface);
      const recvId = stringifyId(entry.received_from);

      // 1. Create/Update Interface Node (Green)
      if (!graph.hasNode(ifaceId)) {
        graph.addNode(ifaceId, { 
          label: `IF: ${ifaceId.slice(-4)}`, 
          size: 10, 
          color: "#2ed573" 
        });
        graph.addEdge("origin", ifaceId, { color: "#2ed573", weight: 2 });
      }

      // 2. Create/Update Data Node
      // Determine color: Gray if direct (hops: 0), Blue if indirect (hops > 0)
      const nodeColor = entry.hops === 0 ? "#747d8c" : "#1e90ff";
      const nodeSize = entry.hops === 0 ? 8 : 6;

      if (!graph.hasNode(nodeId)) {
        graph.addNode(nodeId, { 
          label: nodeId.slice(0, 8), 
          size: nodeSize, 
          color: nodeColor 
        });
      } else {
        // If the node was already added (e.g., as a recvId earlier), 
        // update its color based on its own actual hop data
        graph.setNodeAttribute(nodeId, "color", nodeColor);
        graph.setNodeAttribute(nodeId, "size", nodeSize);
      }

      // 3. Create Relay Node placeholder if it doesn't exist
      // (If we haven't processed the entry for this ID yet, we'll default it to gray)
      if (recvId && !graph.hasNode(recvId)) {
        graph.addNode(recvId, { 
          label: `R: ${recvId.slice(0, 4)}`, 
          size: 8, 
          color: "#747d8c" 
        });
      }
    });

    // PASS 2: Edges
    rawData.forEach((entry) => {
      const nodeId = stringifyId(entry.id);
      const ifaceId = stringifyId(entry.iface);
      const recvId = stringifyId(entry.received_from);

      if (!graph.hasEdge("origin", ifaceId)) {
        graph.addEdge("origin", ifaceId, { color: "#2ed573", size: 2 });
      }

      if (entry.hops === 0) {
        if (!graph.hasEdge(nodeId, ifaceId)) graph.addEdge(nodeId, ifaceId);
      } else if (recvId) {
        if (graph.hasNode(nodeId) && graph.hasNode(recvId)) {
          if (!graph.hasEdge(nodeId, recvId)) graph.addEdge(nodeId, recvId);
        }
        // Chain the relay to the interface to prevent floating
        if (graph.hasNode(recvId) && graph.hasNode(ifaceId) && recvId !== ifaceId) {
          if (!graph.hasEdge(recvId, ifaceId)) graph.addEdge(recvId, ifaceId, { color: "#ffa502", weight: 0.5 });
        }
      }
    });

    // Layout & Render
    graph.nodes().forEach(n => {
      graph.setNodeAttribute(n, "x", Math.random());
      graph.setNodeAttribute(n, "y", Math.random());
    });

    forceAtlas2.assign(graph, { 
      iterations: 200, 
      settings: { gravity: 1.2, scalingRatio: 10, strongGravityMode: true } 
    });

    const renderer = new Sigma(graph, container, {
      labelSize: 11,
      labelWeight: "bold",
      labelColor: { color: "#fff" },
      defaultEdgeType: "line"
    });

    onMount(() => () => renderer.kill());
  });

  return (
    <div style={{ width: "100%", height: "100vh", background: "#1a1a1a" }}>
      <div ref={container} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
