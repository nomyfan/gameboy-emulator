import { globalStyle, style } from "@vanilla-extract/css";
import { textEllipsis, cssVars, px } from "gameboy/styles";

export const list = style({
  gap: 10,
  ...px(10),
  overflowX: "auto",
});

export const listItem = style({
  width: 180,
  flexGrow: 0,
  flexShrink: 0,
  boxShadow: "0 4px 4px rgba(0,0,0,.25)",
  borderRadius: 1,
  boxSizing: "border-box",
  border: `3px solid ${cssVars.colorPrimary}`,
});

export const listItemSelected = style([
  listItem,
  {
    borderColor: cssVars.colorHighlight,
  },
]);

globalStyle(`${listItem} > figure`, {
  height: "100%",
  width: "100%",
  margin: 0,
});

globalStyle(`${listItem} > figure > figcaption`, {
  fontSize: 14,
  fontWeight: 500,
  backgroundColor: cssVars.colorPrimary,
  color: "white",
  padding: 3,
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
  width: "fit-content",
  color: cssVars.colorPrimary,
  border: "none",
});
