import { keyframes, style } from "@vanilla-extract/css";
import { cssVars } from "gameboy/styles";

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
  width: "40%",
  backgroundColor: cssVars.colorBackground,
  animation: `${contentShow} 500ms cubic-bezier(0.16, 1, 0.3, 1)`,
});
