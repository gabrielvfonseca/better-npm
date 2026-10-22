import { Separator as BaseSeparator } from "@base-ui/react/separator"
import * as React from "react"

import { cn } from "../lib/utils"

export type SeparatorProps = Omit<
  React.ComponentProps<typeof BaseSeparator>,
  "decorative" | "ref"
> & {
  decorative?: boolean
}

const Separator = React.forwardRef<HTMLDivElement, SeparatorProps>(
  ({ className, orientation = "horizontal", decorative = true, ...props }, ref) => (
    <BaseSeparator
      ref={ref}
      data-slot="separator"
      orientation={orientation}
      role={decorative ? "none" : "separator"}
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
