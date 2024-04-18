import { globalStyle } from "@vanilla-extract/css";

import * as cssVars from "./styles/vars.css";

globalStyle("body", {
  color: cssVars.colorPrimary,
});
