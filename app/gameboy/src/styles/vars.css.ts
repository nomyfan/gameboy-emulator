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

export const varColorPrimary = createVar();
export const colorPrimary = fallbackVar(varColorPrimary, "#2C2C2C");

export const varColorText = createVar();
export const colorText = fallbackVar(varColorText, "#4B4B4B");

export const varColorAlert = createVar();
export const colorAlert = fallbackVar(varColorAlert, "#DC3545");
export const varColorAlertRgb = createVar();
export const colorAlertRgb = fallbackVar(varColorAlertRgb, "220,53,69");

export const varColorBackground = createVar();
export const colorBackground = fallbackVar(varColorBackground, "#EBEBEB");
export const varColorBackgroundRgb = createVar();
export const colorBackgroundRgb = fallbackVar(
  varColorBackgroundRgb,
  "235,235,235",
);

export const varColorHighlight = createVar();
export const colorHighlight = fallbackVar(varColorHighlight, "#00FFFF");
