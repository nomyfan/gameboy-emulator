import { globalStyle, style } from "@vanilla-extract/css";

import { flexCenter, py, lightShadow, darkShadow } from "../styles";

export const screen = style({
  ...flexCenter(),
  ...py(20),
  backgroundColor: "#3E3C48",
  boxShadow: `${darkShadow("4px 4px 4px")}, ${lightShadow("-4px -4px 4px")}`,
});

globalStyle(`${screen} > canvas`, {
  backgroundColor: "white",
  border: "5px solid black",
});
