import { onMount, createResource, createSignal, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Graph from "graphology";
import Sigma from "sigma";
import FA2Layout from "graphology-layout-forceatlas2/worker";

export default function NodeGraph() {
  let container;
  const [data] = createResource(async () => await invoke("get_graph_data"));
  const [isRunning, setIsRunning] = createSignal(false);
  
  let layout = null;
  let renderer = null;

  const stringifyId = (val) => {
    if (!val) return null;
    if (typeof val === 'string') return val;
    const inner = val.id || val.key || val;
    if (inner && inner.String) return inner.String;
    return (typeof val === 'object') ? (val.tb ? `${val.tb}:${val.id}` : JSON.stringify(val)) : String(val);
  };

  onMount(async () => {
    const rawData = await invoke("get_graph_data");
    if (!rawData || !container) return;

    const graph = new Graph();

    // 1. Setup Origin
    graph.addNode("origin", { label: "Origin", size: 15, color: "#ff4757", x: 0, y: 0, fixed: true });

    const IFACE_RADIUS = 50;
    const OUTER_RADIUS = 300;

    rawData.forEach((entry) => {
      const nodeId = stringifyId(entry.id);
      const ifaceId = stringifyId(entry.iface);
      const recvId = stringifyId(entry.received_from);
    
      // 1. Ensure Interface exists
      if (!graph.hasNode(ifaceId)) {
        const angle = Math.random() * Math.PI * 2;
        graph.addNode(ifaceId, { 
          label: `IF: ${ifaceId.slice(-4)}`, 
          size: 10, color: "#2ed573", 
          x: Math.cos(angle) * 50, y: Math.sin(angle) * 50 
        });
        // REDUCED WEIGHT: from 20 to 5 so it's not a "rigid" pull
        graph.addEdge("origin", ifaceId, { color: "#2ed573", size: 2, weight: 5 });
      }
    
      // 2. Ensure Data Node exists with Label
      const nodeColor = entry.hops === 0 ? "#747d8c" : "#1e90ff";
      if (!graph.hasNode(nodeId)) {
        const angle = Math.random() * Math.PI * 2;
        const r = 300 + (Math.random() * 200);
        graph.addNode(nodeId, { 
          label: nodeId.slice(0, 8), // NOW ALL NODES GET LABELS
          size: 6, color: nodeColor,
          x: Math.cos(angle) * r, y: Math.sin(angle) * r 
        });
      }
    
      // 3. Connect to Interface or Relay
      if (entry.hops === 0) {
        if (!graph.hasEdge(nodeId, ifaceId)) {
          graph.addEdge(nodeId, ifaceId, { weight: 1 });
        }
      } else if (recvId) {
        const rId = stringifyId(recvId);
        // Ensure the relay node exists and HAS A LABEL
        if (!graph.hasNode(rId)) {
          const angle = Math.random() * Math.PI * 2;
          const r = 350 + (Math.random() * 150);
          graph.addNode(rId, { 
            label: rId.slice(0, 8), 
            size: 8,
            color: "#747d8c",
            x: Math.cos(angle) * r,
            y: Math.sin(angle) * r 
          });
        }
        if (!graph.hasEdge(nodeId, rId)) {
          graph.addEdge(nodeId, rId, { weight: 1 });
        }
        // IMPORTANT: Ensure the relay itself is pulled toward the interface
        if (!graph.hasEdge(rId, ifaceId)) {
          graph.addEdge(rId, ifaceId, { color: "#444", weight: 1 });
        }
      }
    });

    // 3. Anchor logic: Prevent Origin from moving
    graph.on('updated', () => {
      graph.setNodeAttribute("origin", "x", 0);
      graph.setNodeAttribute("origin", "y", 0);
    });

    // 4. Initialize Renderer
    renderer = new Sigma(graph, container, {
      labelSize: 11,
      labelColor: { color: "#fff" },
      defaultEdgeType: "line"
    });

    // 5. Initialize Layout (Matching reference style)
    layout = new FA2Layout(graph, {
      settings: {
        // adjustSizes: true,
        // gravity: 0.5,
        // scalingRatio: 40,
        // barnesHutOptimize: true,
        // slowDown: 5,
        // edgeWeightInfluence: 0.2
        adjustSizes: true,        
    
        // 2. Increase global repulsion (the "push" between all nodes)
        scalingRatio: 80,         // Increased from 15 to 40 to spread blue nodes out
        
        // 3. Make the "springs" even softer
        edgeWeightInfluence: 0.1, // Lowered from 0.8 to 0.2. 
                                  // This allows repulsion to win over attraction.
    
        // 4. Prevent the "crush" toward the center
        gravity: 0.05,            // Very low gravity so nodes aren't forced inward
        
        barnesHutOptimize: true,
        slowDown: 0.1,
      }
    });

    // Start simulation automatically
    layout.start();
    setIsRunning(true);
  });

  // 6. Toggle Logic using layout.isRunning() from reference
  const toggleSimulation = () => {
    if (!layout) return;

    if (layout.isRunning()) {
      layout.stop();
      setIsRunning(false);
      // Optional: Reduce CPU further by refreshing renderer once and letting it idle
      renderer.refresh();
    } else {
      layout.start();
      setIsRunning(true);
    }
  };

  onCleanup(() => {
    if (layout) layout.kill();
    if (renderer) renderer.kill();
  });

  return (
    <div style={{ position: "relative", width: "100%", height: "100vh", background: "#1a1a1a" }}>
      <div style={{
        position: "absolute", top: "20px", left: "20px", "z-index": 10,
        background: "rgba(0,0,0,0.8)", padding: "10px", "border-radius": "8px", border: "1px solid #444"
      }}>
        <button 
          onClick={toggleSimulation}
          style={{
            padding: "8px 16px", background: isRunning() ? "#ff4757" : "#2ed573",
            color: "white", border: "none", "border-radius": "4px", cursor: "pointer", "font-weight": "bold"
          }}
        >
          {isRunning() ? "Freeze Layout ⏸" : "Resume Layout ▶"}
        </button>
      </div>
      <div ref={container} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
