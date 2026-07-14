import { Checkbox as BaseCheckbox } from "@base-ui/react/checkbox"
import { CheckIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

export type CheckboxProps = React.ComponentProps<typeof BaseCheckbox.Root>

const Checkbox = React.forwardRef<React.ElementRef<typeof BaseCheckbox.Root>, CheckboxProps>(
  ({ className, ...props }, ref) => (
    <BaseCheckbox.Root
      ref={ref as React.Ref<typeof BaseDialog.Trigger>}
      data-slot="checkbox"
      className={cn(
        "peer size-4 shrink-0 rounded-sm border border-primary shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 data-[checked]:bg-primary data-[checked]:text-primary-foreground data-[indeterminate]:bg-primary data-[indeterminate]:text-primary-foreground",
        className,
      )}
      {...props}
    >
      <BaseCheckbox.Indicator
        data-slot="checkbox-indicator"
        className="flex items-center justify-center text-current transition-none"
      >
        <CheckIcon className="size-3.5" />
      </BaseCheckbox.Indicator>
    </BaseCheckbox.Root>
  ),
)
Checkbox.displayName = "Checkbox"

export { Checkbox }
