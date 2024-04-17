import { style, globalStyle } from "@vanilla-extract/css";

import { rem } from "../../styles";

const size = rem(100);

export const avatar = style({
  height: size,
  width: size,
  border: `${rem(5)} solid white`,
  borderRadius: "50%",
  boxShadow: "0 4px 4px rgba(0,0,0,.25)",
});

globalStyle(`${avatar} > img`, {
  borderRadius: "50%",
});
