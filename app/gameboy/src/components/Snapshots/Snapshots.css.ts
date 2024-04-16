import { style, keyframes, globalStyle } from "@vanilla-extract/css";

import { rem } from "../../styles";
import * as cssVars from "../../styles/vars.css";

const overlayShow = keyframes({
  from: { opacity: 0 },
  to: { opacity: 1 },
});

export const overlay = style({
  position: "fixed",
  top: 0,
  left: 0,
  width: "100%",
  height: "100%",
  background: "rgba(0, 0, 0, 0.75)",
  backdropFilter: "blur(3px)",
  animation: `${overlayShow} 300ms cubic-bezier(0.16, 1, 0.3, 1)`,
});

const contentShow = keyframes({
  from: { transform: "translateX(100%)", opacity: 0 },
  to: { transform: "translateX(0)", opacity: 1 },
});

export const drawer = style({
  position: "fixed",
  right: 0,
  top: 0,
  height: "100vh",
  width: rem(800),
  backgroundColor: cssVars.colorBackground,
  animation: `${contentShow} 500ms cubic-bezier(0.16, 1, 0.3, 1)`,
});

export const snapshotsRoot = style({
  padding: `${rem(30)} ${rem(20)} 0`,
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
  margin: `${rem(20)} 0`,
});

export const itemImage = style({
  height: rem(144 * 1.7),
  width: rem(160 * 1.7),
  flexShrink: 0,
  flexGrow: 0,
});

export const itemDesc = style({
  flex: "1 0 0",
  padding: rem(20),
  fontSize: rem(40),
  display: "flex",
  alignItems: "center",
  color: "white",
  wordBreak: "break-all",
});

export const itemSubDesc = style({
  fontSize: rem(32),
});

//#region Context menu
const menuRadius = 4;
export const menuContent = style({
  backgroundColor: "white",
  borderRadius: menuRadius,
  fontSize: rem(36),
  padding: rem(12),
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
  padding: `${rem(10)} ${rem(20)}`,
});

export const menuItemIcon = style({
  height: rem(48),
  width: rem(48),
  marginRight: rem(10),
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
