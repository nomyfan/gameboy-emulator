import init, { init_panic_hook } from "gb-wasm";
import React from "react";
import ReactDOM from "react-dom/client";

import "./index.css";
import { App } from "./App";

init().then(() => {
  init_panic_hook();
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
});
