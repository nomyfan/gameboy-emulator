import { style } from "@vanilla-extract/css";

import { size, px } from "../styles";

export const button = style({
  borderRadius: "50%",
  backgroundColor: "#9B0757",
  ...size(50),
  boxShadow: "3px 3px 4px rgba(0,0,0,.25)",
});

export const buttonLabel = style({
  width: 45,
  textShadow: "-2px -2px 4px rgba(255,255,255,.25),3px 3px 4px rgba(0,0,0,.25)",
});

export const abButton = style({
  position: "relative",
  transform: "rotate(-25deg) translateY(-12px)",
});

export const buttonGroup = style({
  width: "fit-content",
  display: "flex",
  borderRadius: 50,
  padding: "10px 15px",
});

export const labelGroup = style({
  display: "flex",
  justifyContent: "space-between",
  textAlign: "center",
  ...px(15),
  fontWeight: 600,
  width: "100%",
  position: "absolute",
  marginLeft: 5,
  boxSizing: "border-box",
});
