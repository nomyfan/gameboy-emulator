import * as RadixAvatar from "@radix-ui/react-avatar";
import { FlexBox } from "gameboy/components/flex-box";
import type { ReactNode } from "react";

import * as styles from "./Avatar.css";

export function Avatar(props: { src?: string; fallback?: ReactNode }) {
  return (
    <RadixAvatar.Root className={styles.avatar}>
      <RadixAvatar.Image src={props.src} alt="avatar" />
      <RadixAvatar.Fallback asChild>
        <FlexBox className={styles.fallback} justify="center" align="center">
          {props.fallback}
        </FlexBox>
      </RadixAvatar.Fallback>
    </RadixAvatar.Root>
  );
}
