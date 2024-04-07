import { style } from "@vanilla-extract/css";

import { lightShadowColorVar, darkShadowColorVar } from "./styles/vars.css";

export const app = style({
  minHeight: "100vh",
  backgroundColor: "#C8C4BE",
});

export const appPortrailVars = style({
  vars: {
    [lightShadowColorVar]: "rgba(255,255,255,.25)",
    [darkShadowColorVar]: "rgba(0,0,0,.25)",
  },
});

export const appLandscapeVars = style({
  vars: {
    [lightShadowColorVar]: "rgba(60,62,72,.25)",
    [darkShadowColorVar]: "rgba(0,0,0,.25)",
  },
});
