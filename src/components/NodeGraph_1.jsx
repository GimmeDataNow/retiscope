import { onMount, createEffect, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Graph from "graphology";
import Sigma from "sigma";
import forceAtlas2 from "graphology-layout-forceatlas2";

export default function NodeGraph() {
  let container;
  
  // 1. Fetch data from Rust
  const [data] = createResource(async () => {
    return await invoke("get_graph_data");
  });

  createEffect(() => {
    if (!data() || !container) return;

    const graph = new Graph();

    // Add Central Origin
    graph.addNode("origin", { 
      label: "Origin", 
      size: 15, 
      color: "#ff4757", 
      x: 0, 
      y: 0 
    });

    // data().forEach((entry) => {
    //   const { id, iface, hops, received_from } = entry;

    //   // Ensure iface node exists (Connect to Origin)
    //   if (!graph.hasNode(iface)) {
    //     graph.addNode(iface, { 
    //       label: `Iface: ${iface.substring(0, 6)}`, 
    //       size: 10, 
    //       color: "#2ed573" 
    //     });
    //     graph.addEdge("origin", iface);
    //   }

    //   // Ensure current node exists
    //   if (!graph.hasNode(id)) {
    //     graph.addNode(id, { 
    //       label: `Node: ${id.substring(11, 17)}`, 
    //       size: 7, 
    //       color: "#1e90ff" 
    //     });
    //   }

    //   // Logic: Hops connection
    //   if (hops === 0) {
    //     // Connect directly to the interface
    //     if (!graph.hasEdge(id, iface)) graph.addEdge(id, iface);
    //   } else {
    //     // Connect to the receiver (the previous hop)
    //     // Ensure the received_from node exists to avoid errors
    //     if (!graph.hasNode(received_from)) {
    //         graph.addNode(received_from, { size: 5, color: "#70a1ff" });
    //     }
    //     if (!graph.hasEdge(id, received_from)) graph.addEdge(id, received_from);
    //   }
    // });
    data().forEach((entry) => {
      // Ensure ID is a string if SurrealDB returns it as a RecordId object
      const nodeId = typeof entry.id === 'string' ? entry.id : `${entry.id.tb}:${entry.id.id}`;
      const { iface, hops, received_from } = entry;
    
      // 1. Ensure Origin -> Interface connection
      if (!graph.hasNode(iface)) {
        graph.addNode(iface, { 
          label: `IF: ${iface.slice(0, 6)}`, 
          size: 12, 
          color: "#2ed573",
          x: Math.random(), // Initial random pos for the layout engine
          y: Math.random() 
        });
        if (!graph.hasEdge("origin", iface)) {
          graph.addEdge("origin", iface, { label: "uplink" });
        }
      }
    
      // 2. Add the actual data node
      if (!graph.hasNode(nodeId)) {
        graph.addNode(nodeId, { 
          label: nodeId.split(':')[1]?.slice(0, 6) || nodeId, 
          size: 8, 
          color: "#1e90ff" 
        });
      }
    
      // 3. Connection Logic
      if (hops === 0) {
        // Connect to the specific interface node
        if (!graph.hasEdge(nodeId, iface)) {
          graph.addEdge(nodeId, iface);
        }
      } else if (received_from) {
        // Connect to the node that sent it
        if (!graph.hasNode(received_from)) {
            graph.addNode(received_from, { size: 5, color: "#747d8c" });
        }
        if (!graph.hasEdge(nodeId, received_from)) {
          graph.addEdge(nodeId, received_from);
        }
      }
    });

    // 2. Simple Layout: Spread nodes so they aren't on top of each other
    graph.nodes().forEach((node, i) => {
        if (node === "origin") return;
        graph.setNodeAttribute(node, "x", Math.cos(i) * 100);
        graph.setNodeAttribute(node, "y", Math.sin(i) * 100);
    });

    // Run ForceAtlas2 layout for a few seconds to organize the hairball
    forceAtlas2.assign(graph, { iterations: 50, settings: { gravity: 1 } });

    // 3. Initialize Sigma
    const renderer = new Sigma(graph, container, {
        allowSelectionSync: true,
        renderEdgeLabels: false
    });

    // Cleanup on unmount
    onMount(() => {
      return () => renderer.kill();
    });
  });

  return (
    <div style={{ width: "100%", height: "600px", background: "#2f3542", "border-radius": "8px" }}>
      <div ref={container} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
