import { globalStyle, style } from "@vanilla-extract/css";

import { flexCenter, py } from "../styles";

export const screen = style({
  ...flexCenter(),
  ...py(20),
  backgroundColor: "#3E3C48",
  boxShadow: "4px 4px 4px rgba(0,0,0,.25),-4px -4px 4px rgba(255,255,255,.25)",
});

globalStyle(`${screen} > canvas`, {
  backgroundColor: "white",
  border: "5px solid black",
});
