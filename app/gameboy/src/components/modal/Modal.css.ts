import { keyframes, style } from "@vanilla-extract/css";

import { rem, my } from "../../styles";
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

export const content = style({
  position: "fixed",
  top: 0,
  left: 0,
  width: "100vw",
  height: "100vh",
});

export const container = style({
  maxWidth: "70%",
  minWidth: "35%",
  position: "absolute",
  left: "50%",
  top: "50%",
  transform: "translate(-50%, -50%)",
  backgroundColor: cssVars.colorBackground,
  padding: `${rem(50)} ${rem(60)}`,
  borderRadius: 5,
  color: cssVars.colorPrimary,
});

export const title = style({
  marginBottom: 25,
  marginTop: 0,
  fontSize: 24,
  fontWeight: "bold",
});

export const description = style({
  ...my(25),
  fontSize: 14,
  fontWeight: 500,
});

export const footer = style({
  marginTop: 25,
});
