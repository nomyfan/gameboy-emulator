import type { JoypadButton } from "gb_wasm";

export type IGameBoyButton = keyof typeof JoypadButton;

export type IDirectionButton = Extract<
  IGameBoyButton,
  "Up" | "Down" | "Left" | "Right"
>;

export type IAbButton = Extract<IGameBoyButton, "A" | "B">;

export type IFnButton = Extract<IGameBoyButton, "Start" | "Select">;

export type ExcludeNullValue<T> = { [K in keyof T]: Exclude<T[K], null> };
