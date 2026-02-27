// components/ConnectionLayout.jsx
import "./Connection.css"
import { useGlobal } from "../store/GlobalContext";

import { A } from "@solidjs/router";
import { For } from "solid-js";

export function ConnectionLayout(props) {
  const [state, actions] = useGlobal();

  return (
    <div class="connection-layout-wrapper">
      <div class="connection-layout-wrapper-cosmetic">

        <div class="connections">
          <For each={state.connections}>
            {(item) => (
              /* Concatenating the path dynamically */
              <A class="connections-item" href={`/connections/${item.uuid}`}>
                {item.name}
              </A>
            )}
          </For>
          <div class="connections-item" href="/" >
            +
          </div>

        </div>

          {props.children} 
      </div>
    </div>
  );
}

