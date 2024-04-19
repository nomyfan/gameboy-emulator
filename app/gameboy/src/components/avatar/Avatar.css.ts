import { style, globalStyle } from "@vanilla-extract/css";
import { rem, cssVars } from "gameboy/styles";

const size = rem(100);

export const avatar = style({
  height: size,
  width: size,
  border: `${rem(5)} solid white`,
  borderRadius: "50%",
  boxShadow: "0 4px 4px rgba(0,0,0,.25)",
  display: "block",
});

globalStyle(`${avatar} > img`, {
  borderRadius: "50%",
  height: "100%",
  width: "100%",
  objectFit: "cover",
});

export const fallback = style({
  height: "100%",
  width: "100%",
  fontSize: 14,
  backgroundColor: cssVars.colorPrimary,
  color: "white",
  borderRadius: "50%",
});
