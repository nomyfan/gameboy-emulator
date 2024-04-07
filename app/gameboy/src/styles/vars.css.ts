import { createVar, fallbackVar } from "@vanilla-extract/css";

export const lightShadowColorVar = createVar();
export const lightShadowColor = fallbackVar(
  lightShadowColorVar,
  "rgba(255,255,255,.25)",
);

export const darkShadowColorVar = createVar();
export const darkShadowColor = fallbackVar(
  darkShadowColorVar,
  "rgba(0,0,0,.25)",
);
