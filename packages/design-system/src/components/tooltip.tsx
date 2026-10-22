import { Tooltip as BaseTooltip } from "@base-ui/react/tooltip"
import * as React from "react"

import { cn } from "../lib/utils"

const TooltipDelayContext = React.createContext<{ delay: number; closeDelay: number }>({
  delay: 600,
  closeDelay: 0,
})

const TooltipProvider = BaseTooltip.Provider

interface TooltipProps
  extends Omit<React.ComponentProps<typeof BaseTooltip.Root>, "delay" | "closeDelay"> {
  delay?: number
  closeDelay?: number
}

function Tooltip({ delay = 600, closeDelay = 0, ...props }: TooltipProps) {
  return (
    <TooltipDelayContext.Provider value={{ delay, closeDelay }}>
      <BaseTooltip.Root data-slot="tooltip" {...props} />
    </TooltipDelayContext.Provider>
  )
}
Tooltip.displayName = "Tooltip"

const TooltipTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseTooltip.Trigger>
>(({ className, ...props }, ref) => {
  const { delay, closeDelay } = React.useContext(TooltipDelayContext)
  return (
    <BaseTooltip.Trigger
      ref={ref}
      data-slot="tooltip-trigger"
      delay={delay}
      closeDelay={closeDelay}
      className={className}
      {...props}
    />
  )
})
TooltipTrigger.displayName = "TooltipTrigger"

const TooltipContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseTooltip.Popup> & {
    sideOffset?: number
  }
>(({ className, sideOffset = 4, children, ...props }, ref) => (
  <BaseTooltip.Portal>
    <BaseTooltip.Positioner data-slot="tooltip-positioner" sideOffset={sideOffset} className="z-50">
      <BaseTooltip.Popup
        ref={ref}
        data-slot="tooltip-content"
        className={cn(
          "z-50 overflow-hidden rounded-md bg-primary px-3 py-1.5 text-xs text-primary-foreground shadow-md animate-fade-in",
          className,
        )}
        {...props}
      >
        {children}
      </BaseTooltip.Popup>
    </BaseTooltip.Positioner>
  </BaseTooltip.Portal>
))
TooltipContent.displayName = "TooltipContent"

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider }
