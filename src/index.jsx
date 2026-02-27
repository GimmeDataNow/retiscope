/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Route, Navigate } from "@solidjs/router";
import { Sidebar } from "./components/Sidebar";
import { ConnectionLayout } from "./components/ConnectionLayout.jsx";
import { Connection } from "./components/Connection.jsx";
import { GlobalProvider } from "./store/GlobalContext";
import App from "./App";
import Logview from "./Logview.jsx"

render(
    () => (
        <GlobalProvider>
            <Router root={Sidebar}>
                <Route path="/" component={App} />
                <Route path="/logview"/>
                <Route path="/connections" component={ConnectionLayout}>
                    <Route path="/" component={() => <Navigate href="local" />} />
                    <Route path="/local" component={() => <Connection />} />
                    <Route path="/:id" component={() => <Connection />} />
                </Route>
            </Router>
        </GlobalProvider>
    ),
    document.getElementById("root")
);
