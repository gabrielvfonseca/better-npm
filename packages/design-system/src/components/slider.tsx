import { Slider as BaseSlider } from "@base-ui/react/slider"
import type * as React from "react"

import { cn } from "../lib/utils"

export type SliderProps = React.ComponentProps<typeof BaseSlider.Root>

function Slider({ className, children, ...props }: SliderProps & { children?: React.ReactNode }) {
  return (
    <BaseSlider.Root
      data-slot="slider"
      className={cn(
        "relative flex w-full touch-none select-none items-center data-[orientation=vertical]:h-full data-[orientation=vertical]:min-h-44 data-[orientation=vertical]:w-auto data-[orientation=vertical]:flex-col data-[disabled]:opacity-50",
        className,
      )}
      {...props}
    >
      <BaseSlider.Control
        data-slot="slider-control"
        className="relative flex w-full items-center data-[orientation=vertical]:h-full data-[orientation=vertical]:w-auto"
      >
        <BaseSlider.Track
          data-slot="slider-track"
          className="relative h-1.5 w-full grow overflow-hidden rounded-full bg-primary/20 data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5"
        >
          <BaseSlider.Indicator
            data-slot="slider-indicator"
            className="absolute h-full bg-primary data-[orientation=vertical]:w-full"
          />
        </BaseSlider.Track>
        {children ?? <SliderThumb />}
      </BaseSlider.Control>
    </BaseSlider.Root>
  )
}
Slider.displayName = "Slider"

function SliderThumb(props: React.ComponentProps<typeof BaseSlider.Thumb>) {
  return (
    <BaseSlider.Thumb
      data-slot="slider-thumb"
      className="block size-4 rounded-full border border-primary/50 bg-background shadow transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
      {...props}
    />
  )
}

export { Slider, SliderThumb }
