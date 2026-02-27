import { createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";

const GlobalContext = createContext();

export function GlobalProvider(props) {
  const [state, setState] = createStore({
    connections: [
      { uuid: "local", name: "Local Server" },
      { uuid: "alma", name: "Remote Alma" },
      { uuid: "ulm", name: "Remote Ulm" },
      { uuid: "tret", name: "Remote Tret" },
    ],
    activeWidget: null,
    isSidebarOpen: true
  });

  const store = [
    state,
    {
      addConnection(name, uuid) {
        setState("connections", (c) => [...c, { name, uuid }]);
      },
      setWidget(type) {
        setState("activeWidget", type);
      }
    }
  ];

  return (
    <GlobalContext.Provider value={store}>
      {props.children}
    </GlobalContext.Provider>
  );
}

// Custom hook for easy access
export function useGlobal() { return useContext(GlobalContext); }
