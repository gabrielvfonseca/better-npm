import { cva } from "class-variance-authority"
import { Loader2 } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const spinnerVariants = cva("animate-spin text-muted-foreground", {
  variants: {
    size: {
      sm: "size-4",
      md: "size-6",
      lg: "size-8",
      xl: "size-12",
    },
  },
  defaultVariants: {
    size: "md",
  },
})

export interface SpinnerProps extends React.SVGAttributes<SVGSVGElement> {
  size?: "sm" | "md" | "lg" | "xl"
  label?: string
}

const Spinner = React.forwardRef<SVGSVGElement, SpinnerProps>(
  ({ className, size, label = "Loading", ...props }, ref) => (
    <Loader2
      ref={ref}
      role="status"
      aria-label={label}
      className={cn(spinnerVariants({ size }), className)}
      {...props}
    />
  ),
)
Spinner.displayName = "Spinner"

export { Spinner, spinnerVariants }
