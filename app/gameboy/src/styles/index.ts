import type { CSSProperties } from "@vanilla-extract/css";

import { lightShadowColor, darkShadowColor } from "./vars.css";

export const px = <T extends CSSProperties["paddingLeft"]>(value: T) => ({
  paddingLeft: value,
  paddingRight: value,
});

export const py = <T extends CSSProperties["paddingTop"]>(value: T) => ({
  paddingTop: value,
  paddingBottom: value,
});

export const mx = <T extends CSSProperties["marginLeft"]>(value: T) => ({
  marginLeft: value,
  marginRight: value,
});

export const my = <T extends CSSProperties["marginTop"]>(value: T) => ({
  marginTop: value,
  marginBottom: value,
});

export const size = <T extends CSSProperties["width"]>(value: T) => ({
  width: value,
  height: value,
});

export const flexCenter = () =>
  ({
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
  }) as const;

export const lightShadow = <T extends string>(
  value: T,
): `${T} ${typeof lightShadowColor}` => `${value} ${lightShadowColor}`;

export const darkShadow = <T extends string>(
  value: T,
): `${T} ${typeof darkShadowColor}` => `${value} ${darkShadowColor}`;

export const rem = <T extends number>(px: T): `${T}rem` =>
  (Math.floor(((px * (375 / 1080)) / 18) * 100) / 100 + "rem") as `${T}rem`;

export const textEllipsis = () =>
  ({
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  }) as const;
