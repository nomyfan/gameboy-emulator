import { globalStyle, style } from "@vanilla-extract/css";

import { lightShadow, darkShadow } from "../styles";

export const button = style({
  width: "fit-content",
  transform: "rotate(-25deg)",
});

const shadow = `${lightShadow("-2px -2px 4px")}, ${darkShadow("2px 2px 4px")}`;

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
