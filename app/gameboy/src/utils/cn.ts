export function cn(
  ...classNames: (string | number | null | boolean | undefined)[]
): string;
export function cn() {
  let str = "";
  for (let i = 0; i < arguments.length; i++) {
    // eslint-disable-next-line prefer-rest-params
    const className = arguments[i];
    if (className && typeof className === "string") {
      str += (str && " ") + className;
    }
  }
  return str;
}
