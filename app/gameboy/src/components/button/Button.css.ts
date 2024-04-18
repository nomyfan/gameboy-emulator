import { style } from "@vanilla-extract/css";

import * as cssVars from "../../styles/vars.css";

export const button = style({
  padding: "10px 20px",
  fontWeight: 500,
  borderRadius: 5,
  backgroundColor: "white",
  color: cssVars.colorPrimary,
});

export const buttonPrimary = style({
  backgroundColor: cssVars.colorPrimary,
  color: "white",
});
