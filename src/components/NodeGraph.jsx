import { onMount, createResource, createSignal, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import Graph from "graphology";
import Sigma from "sigma";
import FA2Layout from "graphology-layout-forceatlas2/worker";

export default function NodeGraph() {
  let container;
  const [data] = createResource(async () => await invoke("get_graph_data"));
  const [isRunning, setIsRunning] = createSignal(false);
  const [hoveredNode, setHoveredNode] = createSignal(null);
  
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
  // Fetch both the raw path data and your new Gateway summary
      const [rawData, gatewayData] = await Promise.all([
        invoke("get_graph_data"),
        invoke("get_gateway_summary") // Map this to your new SQL query
      ]);
    
      const graph = new Graph();
      graph.addNode("origin", { label: "Origin", size: 30, color: "#ff4757", x: 0, y: 0, fixed: true });
    
      // 1. Calculate Total Reachable to determine angular weights
      const totalNodes = gatewayData.reduce((sum, g) => sum + g.nodes_reachable, 0);
      
      let currentAngle = 0;
      const gatewayMeta = new Map();
    
      // 2. Map Gateways to specific angular "wedges"
      gatewayData.forEach((gw) => {
        const arcSize = (gw.nodes_reachable / totalNodes) * (Math.PI * 2);
        const centerAngle = currentAngle + (arcSize / 2);
        const ifaceId = stringifyId(gw.primary_iface[0]);
    
        // Store geometry for this branch
        gatewayMeta.set(stringifyId(gw.gateway_address), {
          centerAngle,
          arcSize,
          ifaceId
        });
    
        // Place Interface (Fixed)
        if (!graph.hasNode(ifaceId)) {
          graph.addNode(ifaceId, {
            label: `IF: ${ifaceId.slice(-4)}`,
            x: Math.cos(centerAngle) * 150,
            y: Math.sin(centerAngle) * 150,
            size: 12, color: "#2ed573", fixed: true
          });
          graph.addEdge("origin", ifaceId, { weight: 2 });
        }
    
        // Place Gateway/Relay (if it's not the same as the interface)
        const gwId = stringifyId(gw.gateway_address);
        if (!graph.hasNode(gwId)) {
          const radius = 250 + (gw.min_hops * 50);
          graph.addNode(gwId, {
            label: `GW: ${gwId.slice(0, 8)}`,
            x: Math.cos(centerAngle) * radius,
            y: Math.sin(centerAngle) * radius,
            size: 9, color: "#ffa502" 
          });
          graph.addEdge(gwId, ifaceId, { weight: 5 });
        }
    
        currentAngle += arcSize; // Shift to next wedge
      });
    
      // 3. Populate Leaf Nodes into their designated wedges
      rawData.forEach((entry) => {
        const nodeId = stringifyId(entry.id);
        if (graph.hasNode(nodeId)) return;
    
        const gwId = stringifyId(entry.received_from);
        const meta = gatewayMeta.get(gwId);
    
        if (meta) {
          // Spawn within the wedge assigned to its gateway
          const jitter = (Math.random() - 0.5) * (meta.arcSize * 0.8);
          const radius = 400 + (entry.hops * 60);
          
          graph.addNode(nodeId, {
            label: nodeId.slice(0, 8),
            x: Math.cos(meta.centerAngle + jitter) * radius,
            y: Math.sin(meta.centerAngle + jitter) * radius,
            size: 6, color: "#1e90ff"
          });
          graph.addEdge(nodeId, gwId, { weight: 1 });
        }
      });

    graph.on('updated', () => {
      graph.setNodeAttribute("origin", "x", 0);
      graph.setNodeAttribute("origin", "y", 0);
    });

    renderer = new Sigma(graph, container, {
      labelSize: 11,
      labelColor: { color: "#fff" },
      defaultEdgeType: "line"
    });

    renderer.setSetting("nodeReducer", (node, data) => {
      const res = { ...data };
      const hovered = hoveredNode();
    
      if (hovered) {
        // Is this node the one being hovered, or a direct neighbor?
        const isNeighbor = graph.hasEdge(hovered, node) || graph.hasEdge(node, hovered);
        const isTarget = node === hovered;
    
        if (!isNeighbor && !isTarget) {
          res.label = "";       // Hide label
          res.color = "#333";   // Dim the color
          res.opacity = 0.1;    // Fade out
        } else if (isTarget) {
          res.highlighted = true;
          res.size = data.size * 1.5; // Make the hovered node pop
        }
      }
      return res;
    });
    
    renderer.setSetting("edgeReducer", (edge, data) => {
      const res = { ...data };
      const hovered = hoveredNode();
    
      if (hovered && !graph.hasExtremity(edge, hovered)) {
        res.hidden = true; // Hide edges not connected to the hovered node
      } else if (hovered) {
        res.color = "#2ed573"; // Highlight the active edges
        res.size = 2;
      }
      return res;
    });
    
    // 2. Bind the Events
    renderer.on("enterNode", ({ node }) => {
      setHoveredNode(node);
      renderer.refresh(); // Force a re-draw with the reducers applied
    });
    
    renderer.on("leaveNode", () => {
      setHoveredNode(null);
      renderer.refresh(); // Reset to normal
    });

    layout = new FA2Layout(graph, {
      settings: {
        adjustSizes: true,        
        scalingRatio: 40,
        edgeWeightInfluence: 0.3,
        gravity: 0.05,
        barnesHutOptimize: true,
        slowDown: 0.1,
      }
    });

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
    <div style={{ position: "relative", width: "100%", height: "100vh", background: "#242424" }}>
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
