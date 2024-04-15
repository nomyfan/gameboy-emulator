import { globalStyle, style } from "@vanilla-extract/css";

import { rem } from "../../styles";
import * as cssVars from "../../styles/vars.css";

export const list = style({
  display: "flex",
  gap: rem(20),
  alignItems: "center",
  padding: `${rem(100)} ${rem(50)}`,
  overflow: "auto",
});

export const listItem = style({
  height: rem(500),
  width: rem(500),
  flexGrow: 0,
  flexShrink: 0,
  boxShadow: "0 4px 4px rgba(0,0,0,.25)",
  borderRadius: rem(5),
  boxSizing: "border-box",
});

export const listItemSelected = style({
  border: `${rem(10)} solid ${cssVars.colorHighlight}`,
});

globalStyle(`${listItem} > img`, {
  borderRadius: rem(5),
});

globalStyle(`${listItemSelected} > img`, {
  borderRadius: 0,
});
