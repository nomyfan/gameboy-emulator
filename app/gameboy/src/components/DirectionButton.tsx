import { CSSProperties } from "react";

import { cn } from "../lib/utils";

function Button(props: { className?: string; style?: CSSProperties }) {
  return (
    <button
      className={cn("bg-black h-full w-full rounded-[4px]", props.className)}
      style={props.style}
    />
  );
}

function DirectionButton() {
  return (
    <div
      className={cn("grid bg-[#E4E1DD] rounded-full p-[15px]")}
      style={{
        gridTemplateColumns: "35px 40px 35px",
        gridTemplateRows: "35px 40px 35px",
        boxShadow:
          "inset -4px -4px 4px rgba(255,255,255,.25), inset 4px 4px 4px rgba(0,0,0,.25)",
      }}
    >
      <Button
        key="top"
        className={cn("col-start-2 rounded-b-[0]")}
        style={{
          boxShadow: "-4px -4px 4px rgba(255,255,255,.25)",
        }}
      />
      <Button
        key="left"
        className={cn("row-start-2 rounded-r-[0]")}
        style={{
          boxShadow:
            "0px 4px 4px rgba(0,0,0,.25),-4px -4px 4px rgba(255,255,255,.25)",
        }}
      />
      <div
        key="center"
        className={cn(
          "bg-black row-start-2 col-start-2 flex justify-center items-center",
        )}
      >
        <div
          key="circle"
          className={cn("h-[30px] w-[30px] bg-[#E3E1DD] rounded-full")}
          style={{
            boxShadow:
              "inset -4px -4px 4px rgba(255,255,255,.25), inset 4px 4px 4px rgba(0,0,0,.25)",
          }}
        />
      </div>
      <Button
        key="right"
        className={cn("row-start-2 col-start-3 rounded-l-[0]")}
        style={{
          boxShadow:
            "4px 0px 4px rgba(0,0,0,.25),0px 4px 4px rgba(0,0,0,.25),4px -4px 4px rgba(255,255,255,.25)",
        }}
      />
      <Button
        key="bottom"
        className={cn("row-start-3 col-start-2 rounded-t-[0]")}
        style={{
          boxShadow:
            "0px 4px 4px rgba(0,0,0,.25),-4px 4px 4px rgba(255,255,255,.25)",
        }}
      />
    </div>
  );
}

export { DirectionButton };
