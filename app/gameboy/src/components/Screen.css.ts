import { globalStyle, style } from "@vanilla-extract/css";

import { flexCenter, py } from "../styles";

export const screen = style({
  ...flexCenter(),
  ...py(20),
});

globalStyle(`${screen} > canvas`, {
  backgroundColor: "white",
  border: "5px solid black",
});
