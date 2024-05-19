import { globalStyle, style } from "@vanilla-extract/css";
import { cssVars } from "gameboy/styles";

export const container = style({
  width: "100vw",
  height: "100vh",
  // backgroundColor: cssVars.colorBackground,
  backgroundColor: `rgba(${cssVars.colorBackgroundRgb}, 0.75)`,
  backdropFilter: "blur(20px)",
});

export const tabs = style({
  height: "100%",
  width: "100%",
  display: "flex",
  gap: 20,
});

export const list = style({
  //
});

export const trigger = style({
  display: "block",
  width: "100%",
  textAlign: "left",
  fontSize: "1rem",
  backgroundColor: "transparent",
  borderLeft: "3px solid transparent",
  padding: "6px 12px",
  color: "inherit",
});

globalStyle(`${trigger}[data-state="active"]`, {
  borderLeft: `3px solid ${cssVars.colorPrimary}`,
});

export const content = style({
  flexGrow: 1,
  padding: 6,
  display: "flex",
  flexDirection: "column",
  outline: "none",
});

globalStyle(`${content}[data-state="active"]`, {
  //
});

globalStyle(`${content}[data-state="inactive"]`, {
  display: "none",
});

// ---- slider
export const slider = style({
  position: "relative",
  display: "flex",
  alignItems: "center",
  userSelect: "none",
  touchAction: "none",
  width: 200,
  height: 20,
});

export const track = style({
  position: "relative",
  flexGrow: 1,
  borderRadius: 4,
  height: 4,
  backgroundColor: cssVars.colorPrimary,
});

export const range = style({
  position: "absolute",
  backgroundColor: "white",
  borderRadius: 9999,
  height: "100%",
});

export const thumb = style({
  display: "block",
  width: 20,
  height: 20,
  backgroundColor: "white",
  borderRadius: 10,
});
