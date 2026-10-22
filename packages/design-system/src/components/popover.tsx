import { Popover as BasePopover } from "@base-ui/react/popover"
import * as React from "react"

import { cn } from "../lib/utils"

const Popover = (props: React.ComponentProps<typeof BasePopover.Root>) => (
  <BasePopover.Root data-slot="popover" {...props} />
)
Popover.displayName = "Popover"

const PopoverTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BasePopover.Trigger>
>(({ className, ...props }, ref) => (
  <BasePopover.Trigger ref={ref} data-slot="popover-trigger" className={className} {...props} />
))
PopoverTrigger.displayName = "PopoverTrigger"

const PopoverAnchor = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BasePopover.Positioner>
>(({ className, ...props }, ref) => (
  <BasePopover.Positioner ref={ref} data-slot="popover-anchor" className={className} {...props} />
))
PopoverAnchor.displayName = "PopoverAnchor"

const PopoverContent = React.forwardRef<
  HTMLDivElement,
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
        ref={ref}
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
  HTMLButtonElement,
  React.ComponentProps<typeof BasePopover.Close>
>(({ className, ...props }, ref) => (
  <BasePopover.Close ref={ref} data-slot="popover-close" className={className} {...props} />
))
PopoverClose.displayName = "PopoverClose"

const PopoverArrow = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BasePopover.Arrow>
>(({ className, ...props }, ref) => (
  <BasePopover.Arrow
    ref={ref}
    data-slot="popover-arrow"
    className={cn("fill-popover", className)}
    {...props}
  />
))
PopoverArrow.displayName = "PopoverArrow"

export { Popover, PopoverTrigger, PopoverAnchor, PopoverContent, PopoverClose, PopoverArrow }
