import { style, globalStyle } from "@vanilla-extract/css";
import { cssVars } from "gameboy/styles";

export const snapshotsRoot = style({
  padding: "10px 8px 0",
  display: "flex",
  flexDirection: "column",
  height: "100%",
  boxSizing: "border-box",
});

export const header = style({
  fontSize: 20,
  margin: 0,
});

export const itemsContainer = style({
  flex: "1 0 0",
  overflowY: "auto",
});

export const item = style({
  display: "flex",
  backgroundColor: cssVars.colorPrimary,
  width: "100%",
  margin: "8px 0",
  borderRadius: 2,
});

export const itemImage = style({
  width: 160 * 0.6,
  height: 144 * 0.6,
  flexShrink: 0,
  flexGrow: 0,
  borderRadius: "2px 0 0 2px",
});

export const itemDesc = style({
  flex: "1 0 0",
  padding: 6,
  fontSize: 14,
  display: "flex",
  alignItems: "center",
  color: "white",
  wordBreak: "break-all",
});

export const itemSubDesc = style({
  fontSize: 12,
});

//#region Context menu
const menuRadius = 4;
export const menuContent = style({
  backgroundColor: "white",
  borderRadius: menuRadius,
  fontSize: 12,
  padding: 4,
  boxShadow:
    "0px 10px 38px -10px rgba(22, 23, 24, 0.35), 0px 10px 20px -15px rgba(22, 23, 24, 0.2)",
  lineHeight: 1,
  color: cssVars.colorPrimary,
});

export const menuItem = style({
  minWidth: 150,
  backgroundColor: "white",
  display: "flex",
  alignItems: "center",
  borderRadius: menuRadius,
  outline: "none",
  padding: "5px 10px",
});

export const menuItemIcon = style({
  width: 16,
  height: 16,
  marginRight: 4,
});

globalStyle(`${menuItem}[data-highlighted]`, {
  backgroundColor: cssVars.colorPrimary,
  color: "white",
});

export const menuItemAlert = style({
  color: cssVars.colorAlert,
});

globalStyle(`${menuItemAlert}[data-highlighted]`, {
  backgroundColor: cssVars.colorAlert,
  color: "white",
});
//#endregion
