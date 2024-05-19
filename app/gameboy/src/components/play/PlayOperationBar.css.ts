import { style } from "@vanilla-extract/css";

export const container = style({
  position: "fixed",
  bottom: 10,
  width: "100%",
  display: "flex",
  justifyContent: "center",
});

export const barBackground = style({
  backgroundColor: "rgb(255 255 255 / 30%)",
  backdropFilter: "blur(10px)",
  padding: "8px 30px 8px 12px",
  borderRadius: "10px",
  position: "relative",
});

export const collapseButton = style({
  position: "absolute",
  top: "50%",
  right: 6,
  transform: "translateY(-50%)",
});

export const expandButton = style({
  position: "fixed",
  bottom: 0,
  left: 0,
  right: 0,
  margin: "auto",
  transform: "rotate(180deg)",
});
