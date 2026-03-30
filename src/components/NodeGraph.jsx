import { onMount, onCleanup, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
// import G6 from "@antv/g6";
import * as G6 from "@antv/g6";

export default function ReticulumGraph() {
  let container;
  let graph;

  // We fetch the "Smart" graph data we built in the previous Surreal queries
  const [data] = createResource(async () => await invoke("get_graph_data"));

  onMount(() => {
    // 1. Initialize Graph
    graph = new G6.Graph({
      container: container,
      width: container.scrollWidth,
      height: container.scrollHeight || 800,
      fitView: true,
      layout: {
        type: 'radial',
        unitRadius: 180,        // Distance between rings
        preventOverlap: true,
        maxIteration: 1000,
        sortBy: 'hops',         // Keep rings sorted by hops
        strictRadial: true,     // Force nodes onto the ring lines
      },
      defaultNode: {
        size: 20,
        style: {
          fill: '#1e90ff',
          stroke: '#fff',
          lineWidth: 1,
        },
        labelCfg: {
          style: { fill: '#fff', fontSize: 10 }
        }
      },
      defaultEdge: {
        style: {
          stroke: '#444',
          lineWidth: 1,
          opacity: 0.6,
        },
      },
      modes: {
        default: ['drag-canvas', 'zoom-canvas', 'drag-node'],
      },
    });

    // 2. React to data changes
    if (data()) {
      renderGraph(data());
    }
  });

  const renderGraph = (raw) => {
    // Transform SurrealDB data into G6 format { nodes: [], edges: [] }
    const nodes = raw.map(n => ({
      id: n.id,
      label: n.id.slice(-8), // Short hex for label
      hops: n.hops,
      // Color coding by role
      style: {
        fill: n.node_type === 'origin' ? '#ff4757' : 
              n.node_type === 'neighbor' ? '#2ed573' : '#1e90ff'
      }
    }));

    const edges = raw
      .filter(n => n.parent) // Only nodes with a parent get an edge
      .map(n => ({
        source: n.parent,
        target: n.id
      }));

    graph.data({ nodes, edges });
    graph.render();
  };

  onCleanup(() => {
    if (graph) graph.destroy();
  });

  return (
    <div style={{ width: "100%", height: "100vh", background: "#1a1a1a" }}>
      <div ref={container} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
