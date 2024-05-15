import { style, createVar, globalStyle, keyframes } from "@vanilla-extract/css";
import { darkShadow, px, py } from "gameboy/styles";

const viewportPaddingY = createVar();
const viewportPaddingX = createVar();

export const viewport = style({
  vars: {
    [viewportPaddingX]: "15px",
    [viewportPaddingY]: "6px",
  },
  position: "fixed",
  left: 0,
  right: 0,
  top: 0,
  margin: "0 auto",
  width: "fit-content",
  listStyle: "none",
  outline: "none",
  zIndex: 2147483647,
  ...px(viewportPaddingX),
  ...py(viewportPaddingY),
  display: "flex",
  flexDirection: "column-reverse",
  gap: 6,
});

const slideInKeyframes = keyframes({
  from: {
    transform: `translateY(calc(-100% - ${viewportPaddingY}))`,
  },
  to: {
    transform: "translateY(0)",
  },
});

globalStyle(`${viewport} > [data-state=open]`, {
  backgroundColor: "white",
  animation: `${slideInKeyframes} 350ms cubic-bezier(0.16, 1, 0.3, 1)`,
  display: "flex",
  alignItems: "center",
  padding: "8px 16px",
  borderRadius: 2,
  boxShadow: darkShadow("0px 0px 2px"),
});

globalStyle(`${viewport} > [data-swipe=move]`, {
  transform: "translateX(var(--radix-toast-swipe-move-x))",
});

globalStyle(`${viewport} > [data-swipe=cancel]`, {
  transform: "translateX(0)",
  transition: "transform 250ms ease-out",
});

export const close = style({
  borderRadius: 2,
  marginLeft: 6,
  padding: "4px 6px",
  fontSize: "0.9em",
  fontWeight: "bolder",
  selectors: {
    "&:hover": {
      backgroundColor: "#e5e5e5",
    },
  },
});
