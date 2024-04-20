import { style } from "@vanilla-extract/css";
import { size, lightShadow, darkShadow } from "gameboy/styles";

export const button = style({
  borderRadius: "50%",
  backgroundColor: "#9B0757",
  ...size(50),
  boxShadow: darkShadow("3px 3px 4px"),
});

export const buttonLabel = style({
  width: 45,
  textShadow: `${lightShadow("-2px -2px 4px")}, ${darkShadow("3px 3px 4px")}`,
});

export const abButton = style({
  fontWeight: "bold",
});

export const buttonGroup = style({
  position: "relative",
  display: "flex",
  width: "fit-content",
  borderRadius: 50,
});

export const labelGroup = style({
  display: "flex",
  justifyContent: "space-between",
  textAlign: "center",
  width: "100%",
  position: "absolute",
  bottom: -30,
});
