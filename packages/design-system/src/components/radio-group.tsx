import { Radio as BaseRadio } from "@base-ui/react/radio"
import { RadioGroup as BaseRadioGroup } from "@base-ui/react/radio-group"
import { CircleIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const RadioGroup = React.forwardRef<
  React.ElementRef<typeof BaseRadioGroup>,
  React.ComponentProps<typeof BaseRadioGroup>
>(({ className, ...props }, ref) => (
  <BaseRadioGroup
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="radio-group"
    className={cn("grid gap-2", className)}
    {...props}
  />
))
RadioGroup.displayName = "RadioGroup"

type RadioGroupItemProps = React.ComponentProps<typeof BaseRadio.Root> & {
  value: NonNullable<React.ComponentProps<typeof BaseRadio.Root>["value"]>
}

const RadioGroupItem = React.forwardRef<
  React.ElementRef<typeof BaseRadio.Root>,
  RadioGroupItemProps
>(({ className, value, ...props }, ref) => (
  <BaseRadio.Root
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    value={value}
    data-slot="radio-group-item"
    className={cn(
      "aspect-square size-4 shrink-0 rounded-full border border-primary text-primary shadow focus:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 data-[checked]:border-primary",
      className,
    )}
    {...props}
  >
    <BaseRadio.Indicator
      data-slot="radio-group-indicator"
      className="flex items-center justify-center"
    >
      <CircleIcon className="size-2.5 fill-primary text-primary" />
    </BaseRadio.Indicator>
  </BaseRadio.Root>
))
RadioGroupItem.displayName = "RadioGroupItem"

export { RadioGroup, RadioGroupItem }
