import { style } from "@vanilla-extract/css";
import { cssVars } from "gameboy/styles";

export const button = style({
  padding: "8px 16px",
  fontWeight: 500,
  borderRadius: 5,
  backgroundColor: "white",
  color: cssVars.colorText,
});

export const buttonPrimary = style([
  button,
  {
    backgroundColor: cssVars.colorPrimary,
    color: "white",
  },
]);
