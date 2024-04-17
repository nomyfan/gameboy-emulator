import { globalStyle, style } from "@vanilla-extract/css";

import { flexCenter } from "../styles";

export const screen = style({
  ...flexCenter(),
});

globalStyle(`${screen} > canvas`, {
  backgroundColor: "white",
  border: "5px solid black",
});
