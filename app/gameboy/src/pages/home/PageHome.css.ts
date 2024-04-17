import { style } from "@vanilla-extract/css";

import { rem } from "../../styles";
import * as vars from "../../styles/vars.css";

export const home = style({
  backgroundColor: vars.colorBackground,
  height: "100vh",
  width: "100vw",
  display: "flex",
  flexDirection: "column",
});

export const statusBar = style({
  padding: `${rem(30)} ${rem(50)}`,
});

export const gameList = style({
  flexGrow: 1,
  flexShrink: 0,
});

export const operationBar = style({
  height: rem(220),
});
