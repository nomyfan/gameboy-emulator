import type { CSSProperties, HTMLAttributes } from "react";
import { createElement } from "react";

export function FlexBox<T>(
  props: HTMLAttributes<T> & {
    element?: string;
    direction?: CSSProperties["flexDirection"];
    align?: CSSProperties["alignItems"];
    justify?: CSSProperties["justifyContent"];
    grow?: CSSProperties["flexGrow"];
    shrink?: CSSProperties["flexShrink"];
    gap?: CSSProperties["gap"];
  },
) {
  const {
    children,
    style: styleProp,
    element,
    direction,
    align,
    justify,
    grow,
    shrink,
    gap,
    ...restProps
  } = props;

  const style: CSSProperties = {
    display: "flex",
    flexDirection: direction,
    alignItems: align,
    justifyContent: justify,
    flexGrow: grow,
    flexShrink: shrink,
    gap,
    ...styleProp,
  };

  const Type = element ?? "div";

  return createElement(Type, { style, ...restProps }, children);
}
