import { style } from "@vanilla-extract/css";

import { lightShadowColorVar, darkShadowColorVar } from "./styles/vars.css";

export const app = style({
  minHeight: "100vh",
  backgroundColor: "#C8C4BE",
  vars: {
    [lightShadowColorVar]: "rgba(255,255,255,.25)",
    [darkShadowColorVar]: "rgba(0,0,0,.25)",
  },
});
