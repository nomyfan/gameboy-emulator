import { createElement, CSSProperties, PropsWithChildren } from "react";

export function FlexBox(
  props: PropsWithChildren<{
    className?: string;
    style?: CSSProperties;
    element?: string;
    direction?: CSSProperties["flexDirection"];
    align?: CSSProperties["alignItems"];
    justify?: CSSProperties["justifyContent"];
    grow?: CSSProperties["flexGrow"];
    shrink?: CSSProperties["flexShrink"];
  }>,
) {
  const style: CSSProperties = {
    display: "flex",
    flexDirection: props.direction,
    alignItems: props.align,
    justifyContent: props.justify,
    flexGrow: props.grow,
    flexShrink: props.shrink,
    ...props.style,
  };

  const Type = props.element ?? "div";

  return createElement(
    Type,
    { className: props.className, style },
    props.children,
  );
}
