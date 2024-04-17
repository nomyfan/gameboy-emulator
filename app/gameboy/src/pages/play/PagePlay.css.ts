import { style } from "@vanilla-extract/css";

import * as cssVars from "../../styles/vars.css";

export const root = style({
  backgroundColor: cssVars.colorBackground,
});

export const side = style({
  flexBasis: 0,
  flexGrow: 1,
  flexShrink: 0,
});

const sideSpacing = "20px";
export const leftSide = style({
  padding: `${sideSpacing} ${sideSpacing} 0 0`,
});

export const rightSide = style({
  padding: `${sideSpacing} 0 0 ${sideSpacing}`,
});

export const screen = style({
  flexShrink: 0,
});
