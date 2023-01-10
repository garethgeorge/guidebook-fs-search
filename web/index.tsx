import * as React from "react";
import { render } from "react-dom";

const App = () => <div>Hey!</div>;

const el = document.querySelector("#app");
render(<App />, el);
