import type { CSSProperties } from "@vanilla-extract/css";

export const px = (value: CSSProperties["paddingLeft"]) => ({
  paddingLeft: value,
  paddingRight: value,
});

export const py = (value: CSSProperties["paddingTop"]) => ({
  paddingTop: value,
  paddingBottom: value,
});

export const size = (value: CSSProperties["width"]) => ({
  width: value,
  height: value,
});

export const flexCenter = () => ({
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
});
