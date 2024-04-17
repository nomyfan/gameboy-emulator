import { style } from "@vanilla-extract/css";

import { flexCenter, size } from "../styles";

export const button = style({
  backgroundColor: "black",
  height: "100%",
  width: "100%",
  borderRadius: 4,
});

export const directionButton = style({
  display: "grid",
  gridTemplateColumns: "44px 40px 44px",
  gridTemplateRows: "43px 40px 43px",
});

export const center = style({
  backgroundColor: "black",
  gridRowStart: 2,
  gridColumnStart: 2,
  ...flexCenter(),
});

export const circle = style({
  selectors: {
    [`${center} &`]: {
      ...size(30),
      borderRadius: "50%",
      backgroundColor: "#E3E1DD",
      boxShadow:
        "inset -4px -4px 4px rgba(255,255,255,.25), inset 4px 4px 4px rgba(0,0,0,.25)",
    },
  },
});
