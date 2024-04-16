import { globalStyle, style } from "@vanilla-extract/css";

import { rem } from "../../styles";
import * as cssVars from "../../styles/vars.css";

export const bar = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

export const barItem = style({
  color: cssVars.colorPrimary,
  backgroundColor: "white",
  borderRadius: "50%",
  width: rem(100),
  height: rem(100),
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  margin: `0 ${rem(10)}`,
});

export const barItemAlert = style({
  color: cssVars.colorAlert,
});

globalStyle(`${barItem} > svg`, {
  width: rem(72),
  height: rem(72),
});

export const separator = style({
  width: 2,
  height: rem(50),
  backgroundColor: "whitesmoke",
  margin: "0 5px",
});
