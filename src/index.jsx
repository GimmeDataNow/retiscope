/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import App from "./App";
import Logview from "./Logview.jsx"
import { Sidebar } from "./components/Sidebar";

render(
    () => (
        <Router root={Sidebar}>
            <Route path="/" component={App} />
            <Route path="/logview" component={Logview} />
        </Router>
    ),
    document.getElementById("root")
);
