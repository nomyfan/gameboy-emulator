import { default as IconChecked } from "./checked.svg?react";
import { default as IconIndeterminate } from "./indeterminate.svg?react";
import { default as IconUnchecked } from "./unchecked.svg?react";

export { IconChecked, IconUnchecked, IconIndeterminate };

export function IconCheck(
  props: Parameters<typeof IconChecked>[0] & {
    checked: boolean | "indeterminate";
  },
) {
  const { checked, ...restProps } = props;
  if (checked === "indeterminate") {
    return <IconIndeterminate {...restProps} />;
  }
  if (checked) {
    return <IconChecked {...restProps} />;
  }
  return <IconUnchecked {...restProps} />;
}
