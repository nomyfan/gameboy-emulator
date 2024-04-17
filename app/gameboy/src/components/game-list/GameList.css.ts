import { globalStyle, style } from "@vanilla-extract/css";

import { rem, textEllipsis } from "../../styles";
import * as cssVars from "../../styles/vars.css";

export const list = style({
  display: "flex",
  gap: rem(20),
  alignItems: "center",
  padding: `${rem(100)} ${rem(50)}`,
  overflow: "auto",
});

export const listItem = style({
  width: rem(500),
  flexGrow: 0,
  flexShrink: 0,
  boxShadow: "0 4px 4px rgba(0,0,0,.25)",
  borderRadius: rem(5),
  boxSizing: "border-box",
  border: `${rem(10)} solid ${cssVars.colorPrimary}`,
});

export const listItemSelected = style({
  border: `${rem(10)} solid ${cssVars.colorHighlight}`,
});

globalStyle(`${listItem} > figure`, {
  height: "100%",
  width: "100%",
  margin: 0,
});

globalStyle(`${listItem} > figure > figcaption`, {
  fontSize: rem(40),
  fontWeight: 500,
  backgroundColor: cssVars.colorPrimary,
  color: "white",
  padding: rem(10),
  ...textEllipsis(),
});

globalStyle(`${listItem} > figure > img`, {
  width: "100%",
  objectFit: "cover",
  borderRadius: 0,
  verticalAlign: "top",
});

export const placeholderItem = style({
  boxShadow: "none",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  minWidth: rem(500),
  width: "fit-content",
  color: cssVars.colorPrimary,
  border: "none",
});
