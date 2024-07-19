import type { JoypadButton } from "gb_wasm";

export type IncludeFrom<T, U extends T> = Extract<T, U>;

export type PartialSome<T, K extends keyof T> = Partial<Pick<T, K>> &
  Omit<T, K>;

export type RequiredSome<T, K extends keyof T> = Partial<T> &
  Required<Pick<T, K>>;

export type IGameBoyButton = keyof typeof JoypadButton;

export type IDirectionButton = Extract<
  IGameBoyButton,
  "Up" | "Down" | "Left" | "Right"
>;

export type IAbButton = Extract<IGameBoyButton, "A" | "B">;

export type IFnButton = Extract<IGameBoyButton, "Start" | "Select">;
