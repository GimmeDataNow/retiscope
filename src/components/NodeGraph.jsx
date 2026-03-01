import { onMount, onCleanup } from 'solid-js';
import Graph from 'graphology';
import Sigma from 'sigma';
import FA2Layout from 'graphology-layout-forceatlas2/worker';
import ForceAtlas2 from 'graphology-layout-forceatlas2';

function NetworkGraph(props) {
  let container;
  let renderer;
  let graph;

  onMount(() => {
    graph = new Graph({ type: 'directed' });
    buildGraph(props.nodes, graph);
    
    ForceAtlas2.assign(graph, {
      iterations: 200,
      settings: {
        gravity: 1,
        scalingRatio: 10,
        strongGravityMode: true,
      }
    });

    renderer = new Sigma(graph, container, {
      labelRenderedSizeThreshold: 6,
      hideEdgesOnMove: true,
      labelDensity: 0.07,
    });
  });

  onCleanup(() => renderer?.kill());

  return <div ref={container} style={{ width: '100%', height: '100vh' }} />;
}

export default NetworkGraph;
