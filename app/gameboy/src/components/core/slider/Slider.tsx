import type { SliderProps } from "@radix-ui/react-slider";
import {
  SliderRange,
  SliderThumb,
  SliderTrack,
  Slider as PrimitiveSlider,
} from "@radix-ui/react-slider";
import { cn } from "gameboy/utils/cn";

export function Slider(props: SliderProps) {
  return (
    <PrimitiveSlider
      {...props}
      className={cn(
        "relative flex items-center select-none touch-none h-5",
        props.className,
      )}
    >
      <SliderTrack className="relative flex-grow rounded h-1 bg-primary">
        <SliderRange className="absolute bg-white rounded h-full" />
      </SliderTrack>
      <SliderThumb className="block h-5 w-5 bg-white rounded-full" />
    </PrimitiveSlider>
  );
}
