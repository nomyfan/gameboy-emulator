import { globalStyle, style } from "@vanilla-extract/css";

export const button = style({
  width: "fit-content",
  transform: "rotate(-25deg)",
});

const shadow =
  "-2px -2px 4px rgba(255,255,255,.25), 2px 2px 4px rgba(0,0,0,.25)";

globalStyle(`${button} > button`, {
  height: 15,
  width: 65,
  backgroundColor: "#9F9AAF",
  display: "block",
  borderRadius: 4,
  boxShadow: shadow,
});

globalStyle(`${button} > label`, {
  fontWeight: 600,
  fontSize: 12,
  display: "block",
  width: "100%",
  textAlign: "center",
  textShadow: shadow,
  lineHeight: 1.5,
});

export const fnButton = style({
  display: "flex",
  justifyContent: "center",
});
