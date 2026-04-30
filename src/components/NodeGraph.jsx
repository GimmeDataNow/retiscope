import {
  createSignal,
  createEffect,
  createMemo,
  createResource,
  onMount,
  onCleanup,
} from "solid-js";
import * as G6 from "@antv/g6";
import { invoke } from "@tauri-apps/api/core";
import "./NodeGraph.css";

function ageHours(timestamp) {
  if (!timestamp) return 0;
  const ms = Date.now() - new Date(timestamp).getTime();
  return ms / (1000 * 60 * 60);
}

// Interpolate between two hex colors by t (0–1)
function lerpColor(hex1, hex2, t) {
  const parse = (h) => [
    parseInt(h.slice(1, 3), 16),
    parseInt(h.slice(3, 5), 16),
    parseInt(h.slice(5, 7), 16),
  ];
  const [r1, g1, b1] = parse(hex1);
  const [r2, g2, b2] = parse(hex2);
  const r = Math.round(r1 + (r2 - r1) * t);
  const g = Math.round(g1 + (g2 - g1) * t);
  const b = Math.round(b1 + (b2 - b1) * t);
  return `rgb(${r},${g},${b})`;
}

// Map hops → node size (tweak min/max to taste)
function hopsToSize(hops) {
  const min = 12, max = 40, maxHops = 10;
  const clamped = Math.min(hops ?? 0, maxHops);
  return min + (max - min) * (clamped / maxHops);
}

const NodeGraph = () => {
  /* ---------- refs & signals ---------- */
  const [container, setContainer] = createSignal(null); // <div ref={setContainer} />
  const [graph, setGraph] = createSignal(null);        // G6 instance

  /* ---------- fetch raw announces ---------- */
  const [annos] = createResource(async () => {
    return await invoke("fetch_announces_db");
  });

  /* ---------- build graph data ---------- */
  const graphData = createMemo(() => {
    const data = annos();
    if (!data) return { nodes: [], edges: [] };
  
    const nodeMap = new Map();
    const edges = [];
  
    data.forEach((ann) => {
      const destId = ann.destination;
      const transId = ann.transport_node;
  
      if (!nodeMap.has(destId)) {
        const age = ageHours(ann.timestamp);
        const MAX_AGE_HOURS = 24;
  
        // t=0 → fresh blue, t=1 → fully gray
        const t = Math.min(age / MAX_AGE_HOURS, 1);
        const fill = lerpColor("#2196f3", "#9e9e9e", t);
        const opacity = 1 - t * 0.6; // fade to 40% opacity at max age
  
        nodeMap.set(destId, {
          id: destId,
          label: destId.slice(-8),
          hops: ann.hops ?? 0,
          node_type: "destination",
          timestamp: ann.timestamp,
          style: {
            fill,
            opacity,
            size: hopsToSize(ann.hops),
            stroke: "#1565c0",
            lineWidth: 1,
          },
        });
      }
  
      if (transId && !nodeMap.has(transId)) {
        nodeMap.set(transId, {
          id: transId,
          label: transId.slice(-8),
          hops: 0,
          node_type: "neighbor",
          style: {
            fill: "#4caf50",
            stroke: "#2e7d32",
            size: 40,         // transit nodes are a fixed smaller size
            lineWidth: 1.5,
          },
        });
      }
  
      if (transId) {
        edges.push({
          source: transId,
          target: destId,
          style: { stroke: "#a0a0a0", lineWidth: 1 },
        });
      }
    });
  
    return { nodes: Array.from(nodeMap.values()), edges };
  });

  /* ---------- create G6 graph on mount ---------- */
  onMount(() => {

    const g = new G6.Graph({
      container: container(),
      animation: false,
      autoFit: "view",
      behaviors: ["drag-canvas", "zoom-canvas", "drag-element"],
      layout: {
        type: "force",
        // linkDistance: 100,
        // maxIteration: 300,
        animated: true,
        interval: 0.02,
        linkDistance: 100,
        nodeStrength: -30,    // repulsion between nodes (more negative = more spread)
        edgeStrength: 0.5,    // how strongly edges pull nodes together
        collideStrength: 0.8, // prevents node overlap
        alpha: 0.1,           // initial simulation energy (0–1)
        alphaDecay: 0.05,     // how fast simulation cools down (lower = runs longer)
        alphaMin: 0.001,      // stops when alpha drops below this
      },
      node: {
        style: (model) => ({
          ...model.style,                    // spread per-node style from data
          labelText: model.label,
          labelFill: model.style?.opacity < 0.6 ? "#aaa" : "#333", // dim label too
          labelFontSize: 11,
          labelPlacement: "bottom",
        }),
      },
      edge: {
        style: (model) => ({
          ...model.style,
        }),
      },
    });

    setGraph(g);

    const onResize = () => {
      g.fitView();
    };
  });

  const [simulating, setSimulating] = createSignal(false);

  const toggleSim = () => {
    const g = graph();
    if (!g) return;
    if (simulating()) {
      g.stopLayout();
      setSimulating(false);
    } else {
      g.layout();
      setSimulating(true);
    }
  };

  /* ---------- update graph when data changes ---------- */
  createEffect(() => {
    const g = graph();
    const data = graphData();
    if (!g || !data.nodes.length) return;
    g.setData(data);
    g.render().then(() => g.fitView());
  });

  /* ---------- cleanup ---------- */
  onCleanup(() => {
    const g = graph();
    if (g) g.destroy();
  });

  /* ---------- render ---------- */
  // return <div class="node-graph" ref={setContainer} />;
  return (
  <>
    <div class="node-graph" ref={setContainer} />
    <button
      onClick={toggleSim}
      style={{
        position: "absolute",
        bottom: "16px",
        right: "16px",
        padding: "8px 16px",
        background: simulating() ? "#ef5350" : "#4caf50",
        color: "#fff",
        border: "none",
        borderRadius: "6px",
        cursor: "pointer",
        "font-size": "13px",
      }}
    >
      {simulating() ? "Stop" : "Simulate"}
    </button>
  </>
);
};

export default NodeGraph;
