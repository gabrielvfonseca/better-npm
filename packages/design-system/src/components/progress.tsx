import { Progress as BaseProgress } from "@base-ui/react/progress"
import * as React from "react"

import { cn } from "../lib/utils"

export type ProgressProps = React.ComponentProps<typeof BaseProgress.Root>

const Progress = React.forwardRef<HTMLDivElement, ProgressProps>(({ className, ...props }, ref) => (
  <BaseProgress.Root
    ref={ref}
    data-slot="progress"
    className={cn("relative flex w-full overflow-hidden rounded-full bg-primary/20", className)}
    {...props}
  >
    <BaseProgress.Indicator
      data-slot="progress-indicator"
      className="h-full w-full flex-1 bg-primary transition-all duration-500 data-[complete]:bg-primary data-[indeterminate]:w-1/2 data-[indeterminate]:animate-pulse"
    />
  </BaseProgress.Root>
))
Progress.displayName = "Progress"

export { Progress }
