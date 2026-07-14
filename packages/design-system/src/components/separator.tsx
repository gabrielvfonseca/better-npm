import { Separator as BaseSeparator } from "@base-ui/react/separator"
import * as React from "react"

import { cn } from "../lib/utils"

export type SeparatorProps = Omit<React.ComponentProps<typeof BaseSeparator>, "decorative"> & {
  decorative?: boolean
}

const Separator = React.forwardRef<React.ElementRef<typeof BaseSeparator>, SeparatorProps>(
  ({ className, orientation = "horizontal", decorative = true, ...props }, ref) => (
    <BaseSeparator
      ref={ref as React.Ref<typeof BaseSeparator>}
      data-slot="separator"
      orientation={orientation}
      decorative={decorative}
      className={cn(
        "bg-border shrink-0",
        orientation === "horizontal" ? "h-px w-full" : "h-full w-px",
        className,
      )}
      {...props}
    />
  ),
)
Separator.displayName = "Separator"

export { Separator }
