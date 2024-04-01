import { ButtonHTMLAttributes, ReactNode } from "react";

import { cn } from "../lib/utils";

function Button(props: {
  className?: string;
  onClick?: ButtonHTMLAttributes<unknown>["onClick"];
}) {
  return (
    <button
      onClick={props.onClick}
      className={cn(
        "bg-[#9B0757] h-[45px] w-[45px] rounded-full",
        props.className,
      )}
      style={{
        boxShadow: "3px 3px 4px rgba(0,0,0,.25)",
      }}
    ></button>
  );
}

function ButtonLabel(props: { className?: string; children?: ReactNode }) {
  return (
    <label
      className={cn("w-[45px]", props.className)}
      style={{
        textShadow:
          "-2px -2px 4px rgba(255,255,255,.25),3px 3px 4px rgba(0,0,0,.25)",
      }}
    >
      {props.children}
    </label>
  );
}

function AbButton(props: { onClick?: (button: "A" | "B") => void }) {
  return (
    <div
      className="relative"
      style={{
        transform: "rotate(-25deg)",
      }}
    >
      <div
        className={cn(
          "bg-[#E4E1DD] w-fit flex rounded-[50px] py-[10px] px-[15px]",
        )}
        style={{
          boxShadow:
            "inset -4px -4px 4px rgba(255,255,255,.25),inset 4px 4px 4px rgba(0,0,0,.25)",
        }}
      >
        <Button
          className={cn("mr-[15px] ")}
          onClick={() => props.onClick?.("B")}
        />
        <Button onClick={() => props.onClick?.("A")} />
      </div>

      <div
        className={cn(
          "flex justify-between px-[15px] text-center font-semibold absolute",
        )}
      >
        <ButtonLabel className="mr-[15px]">B</ButtonLabel>
        <ButtonLabel>A</ButtonLabel>
      </div>
    </div>
  );
}

export { AbButton };
