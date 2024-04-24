import { style } from "@vanilla-extract/css";
import { cssVars } from "gameboy/styles";

export const home = style({
  backgroundColor: cssVars.colorBackground,
  height: "100vh",
  width: "100vw",
  display: "flex",
  flexDirection: "column",
});

export const statusBar = style({
  padding: "10px",
});

export const gameList = style({
  flexGrow: 1,
  flexShrink: 0,
});

export const operationBar = style({
  height: 70,
});
