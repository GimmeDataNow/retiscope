/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import App from "./App";
import Logview from "./Logview.jsx"
import { Sidebar } from "./components/Sidebar";
import { ConnectionLayout } from "./components/ConnectionLayout.jsx";
import { Connection } from "./components/Connection.jsx";

render(
    () => (
        <Router root={Sidebar}>
            <Route path="/" component={App} />
            <Route path="/logview" component={ConnectionLayout}>
                <Route path="/" component={() => <Connection />} />
                <Route path="/a" component={() => <Connection />} />
            </Route>
        </Router>
    ),
    document.getElementById("root")
);
