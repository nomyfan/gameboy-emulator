import transformerVariantGroup from "@unocss/transformer-variant-group";
import { defineConfig, presetIcons, presetUno } from "unocss";

// unocss use css string to define keyframes instead of object.
function defineKeyframes(
  keyframes: Record<string, Record<string, string | number>>,
) {
  return (
    "{" +
    Object.entries(keyframes)
      .map(([stopPoints, properties]) => {
        return `${stopPoints}{${Object.entries(properties)
          .map(([prop, value]) => `${prop}:${value}`)
          .join(";")}}`;
      })
      .join(" ") +
    "}"
  );
}

export default defineConfig({
  presets: [presetIcons(), presetUno()],
  transformers: [transformerVariantGroup()],
  shortcuts: {
    "flex-center": "flex items-center justify-center",
    "absolute-center":
      "absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2",
  },
  theme: {
    colors: {
      primary: "#2c2c2c",
      text: "#4b4b4b",
      alert: "#dc3545",
      bg: "#ebebeb",
      accent: "#00ffff",
    },
    animation: {
      keyframes: {
        scale: defineKeyframes({
          "0%,100%": { transform: "scaleY(0.4)", opacity: 0.75 },
          "50%": { transform: "scaleY(1)", opacity: 1 },
        }),
      },
      duration: {
        scale: "1s",
      },
      counts: {
        scale: "infinite",
      },
      timingFns: {
        scale: "ease-in-out",
      },
    },
  },
});
