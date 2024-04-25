import { style } from "@vanilla-extract/css";
import { darkShadow, flexCenter, lightShadow, size } from "gameboy/styles";

export const directionButton = style({
  display: "grid",
  gridTemplateColumns: "44px 40px 44px",
  gridTemplateRows: "44px 40px 44px",
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

export const button = style({
  backgroundColor: "black",
  height: "100%",
  width: "100%",
  borderRadius: 4,
});

export const buttonTop = style([
  button,
  {
    boxShadow: lightShadow("-4px -4px 4px"),
    gridColumnStart: 2,
    borderBottomLeftRadius: 0,
    borderBottomRightRadius: 0,
  },
]);

export const buttonLeft = style([
  button,
  {
    boxShadow: `${darkShadow("0px 4px 4px")}, ${lightShadow("-4px -4px 4px")}`,
    gridRowStart: 2,
    borderTopRightRadius: 0,
    borderBottomRightRadius: 0,
  },
]);

export const buttonRight = style([
  button,
  {
    boxShadow: `${darkShadow("4px 0px 4px")}, ${darkShadow("0px 4px 4px")}, ${lightShadow("4px -4px 4px")}`,
    gridRowStart: 2,
    gridColumnStart: 3,
    borderTopLeftRadius: 0,
    borderBottomLeftRadius: 0,
  },
]);

export const buttonBottom = style([
  button,
  {
    boxShadow: `${darkShadow("0 4px 4px")}, ${lightShadow("-4px 4px 4px")}`,
    gridRowStart: 3,
    gridColumnStart: 2,
    borderTopLeftRadius: 0,
    borderTopRightRadius: 0,
  },
]);