import { Popover as BasePopover } from "@base-ui/react/popover"
import * as React from "react"

import { cn } from "../lib/utils"

const Popover = (props: React.ComponentProps<typeof BasePopover.Root>) => (
  <BasePopover.Root data-slot="popover" {...props} />
)
Popover.displayName = "Popover"

const PopoverTrigger = React.forwardRef<
  React.ElementRef<typeof BasePopover.Trigger>,
  React.ComponentProps<typeof BasePopover.Trigger>
>(({ className, ...props }, ref) => (
  <BasePopover.Trigger
    ref={ref as React.Ref<typeof BasePopover.Trigger>}
    data-slot="popover-trigger"
    className={className}
    {...props}
  />
))
PopoverTrigger.displayName = "PopoverTrigger"

const PopoverAnchor = React.forwardRef<
  React.ElementRef<typeof BasePopover.Positioner>,
  React.ComponentProps<typeof BasePopover.Positioner>
>(({ className, ...props }, ref) => (
  <BasePopover.Positioner
    ref={ref as React.Ref<typeof BasePopover.Positioner>}
    data-slot="popover-anchor"
    className={className}
    {...props}
  />
))
PopoverAnchor.displayName = "PopoverAnchor"

const PopoverContent = React.forwardRef<
  React.ElementRef<typeof BasePopover.Popup>,
  React.ComponentProps<typeof BasePopover.Popup> & {
    align?: "start" | "center" | "end"
    sideOffset?: number
  }
>(({ className, align = "center", sideOffset = 4, children, ...props }, ref) => (
  <BasePopover.Portal>
    <BasePopover.Positioner
      data-slot="popover-positioner"
      sideOffset={sideOffset}
      align={align}
      className="z-50 outline-none"
    >
      <BasePopover.Popup
        ref={ref as React.Ref<typeof BasePopover.Popup>}
        data-slot="popover-content"
        className={cn(
          "z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-none",
          className,
        )}
        {...props}
      >
        {children}
      </BasePopover.Popup>
    </BasePopover.Positioner>
  </BasePopover.Portal>
))
PopoverContent.displayName = "PopoverContent"

const PopoverClose = React.forwardRef<
  React.ElementRef<typeof BasePopover.Close>,
  React.ComponentProps<typeof BasePopover.Close>
>(({ className, ...props }, ref) => (
  <BasePopover.Close
    ref={ref as React.Ref<typeof BasePopover.Close>}
    data-slot="popover-close"
    className={className}
    {...props}
  />
))
PopoverClose.displayName = "PopoverClose"

const PopoverArrow = React.forwardRef<
  React.ElementRef<typeof BasePopover.Arrow>,
  React.ComponentProps<typeof BasePopover.Arrow>
>(({ className, ...props }, ref) => (
  <BasePopover.Arrow
    ref={ref as React.Ref<typeof BasePopover.Arrow>}
    data-slot="popover-arrow"
    className={cn("fill-popover", className)}
    {...props}
  />
))
PopoverArrow.displayName = "PopoverArrow"

export { Popover, PopoverTrigger, PopoverAnchor, PopoverContent, PopoverClose, PopoverArrow }
