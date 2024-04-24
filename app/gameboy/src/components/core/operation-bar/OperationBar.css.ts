import { globalStyle, style } from "@vanilla-extract/css";
import * as cssVars from "gameboy/styles/vars.css";

export const bar = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

export const barItem = style({
  color: cssVars.colorPrimary,
  backgroundColor: "white",
  borderRadius: "50%",
  height: 36,
  width: 36,
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  margin: `0 5px`,
});

export const barItemAlert = style([
  barItem,
  {
    color: cssVars.colorAlert,
  },
]);

globalStyle(`${barItem} > svg`, {
  width: 24,
  height: 24,
});

export const separator = style({
  width: 2,
  height: 18,
  backgroundColor: "#fff",
  margin: "0 5px",
});
