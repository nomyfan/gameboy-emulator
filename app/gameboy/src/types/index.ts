export type IncludeFrom<T, U extends T> = Extract<T, U>;

export type PartialSome<T, K extends keyof T> = Partial<Pick<T, K>> &
  Omit<T, K>;

export type RequiredSome<T, K extends keyof T> = Partial<T> &
  Required<Pick<T, K>>;

export type IGameBoyButton =
  | "UP"
  | "RIGHT"
  | "DOWN"
  | "LEFT"
  | "A"
  | "B"
  | "START"
  | "SELECT";

export type IDirectionButton = Extract<
  IGameBoyButton,
  "UP" | "DOWN" | "LEFT" | "RIGHT"
>;

export type IAbButton = Extract<IGameBoyButton, "A" | "B">;

export type IFnButton = Extract<IGameBoyButton, "START" | "SELECT">;
