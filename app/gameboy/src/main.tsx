import init, { init_panic_hook, init_log } from "gb-wasm";
import React from "react";
import ReactDOM from "react-dom/client";

import "./index.css";
import { App } from "./App";

function initLog() {
  const params = new URLSearchParams(window.location.search);
  const maxLevel = params.get("log_max_level");
  const filters = params.get("log_filters") || "";
  if (maxLevel) {
    init_log(maxLevel, filters);
  }
}

init().then(() => {
  init_panic_hook();
  initLog();
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
});
