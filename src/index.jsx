/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import App from "./App";
import Logview from "./Logview.jsx"

render(
    () => (
        <Router>
            <Route path="/" component={App} />
            <Route path="/logview" component={Logview} />
        </Router>
    ),
    document.getElementById("root")
);
