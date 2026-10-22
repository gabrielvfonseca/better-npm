import { Select as BaseSelect } from "@base-ui/react/select"
import { CheckIcon, ChevronDownIcon, ChevronUpIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const Select = BaseSelect.Root

function SelectGroup(props: React.ComponentProps<typeof BaseSelect.Group>) {
  return <BaseSelect.Group data-slot="select-group" {...props} />
}

function SelectValue(props: React.ComponentProps<typeof BaseSelect.Value>) {
  return <BaseSelect.Value data-slot="select-value" {...props} />
}

const SelectTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseSelect.Trigger>
>(({ className, children, ...props }, ref) => (
  <BaseSelect.Trigger
    ref={ref}
    data-slot="select-trigger"
    className={cn(
      "flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1 data-[placeholder]:text-muted-foreground",
      className,
    )}
    {...props}
  >
    {children}
    <BaseSelect.Icon className="ml-2 flex size-4 items-center justify-center">
      <ChevronDownIcon className="size-4 opacity-50" />
    </BaseSelect.Icon>
  </BaseSelect.Trigger>
))
SelectTrigger.displayName = "SelectTrigger"

const SelectContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseSelect.Popup> &
    Pick<React.ComponentProps<typeof BaseSelect.Positioner>, "sideOffset" | "align" | "alignOffset">
>(({ className, children, sideOffset = 4, align = "start", ...props }, ref) => (
  <BaseSelect.Portal>
    <BaseSelect.Positioner
      data-slot="select-positioner"
      sideOffset={sideOffset}
      align={align}
      className="z-50 w-(--anchor-width) max-h-[min(24rem,var(--available-height))]"
    >
      <BaseSelect.Popup
        ref={ref}
        data-slot="select-content"
        className={cn(
          "max-h-[min(24rem,var(--available-height))] w-(--anchor-width) min-w-32 overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md",
          className,
        )}
        {...props}
      >
        <BaseSelect.List
          data-slot="select-list"
          className="p-1 overflow-y-auto max-h-[min(24rem,var(--available-height))]"
        >
          {children}
        </BaseSelect.List>
      </BaseSelect.Popup>
    </BaseSelect.Positioner>
  </BaseSelect.Portal>
))
SelectContent.displayName = "SelectContent"

const SelectLabel = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseSelect.GroupLabel>
>(({ className, ...props }, ref) => (
  <BaseSelect.GroupLabel
    ref={ref}
    data-slot="select-label"
    className={cn("px-2 py-1.5 text-xs font-medium text-muted-foreground", className)}
    {...props}
  />
))
SelectLabel.displayName = "SelectLabel"

const SelectItem = React.forwardRef<HTMLDivElement, React.ComponentProps<typeof BaseSelect.Item>>(
  ({ className, children, ...props }, ref) => (
    <BaseSelect.Item
      ref={ref}
      data-slot="select-item"
      className={cn(
        "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pe-2 ps-8 text-sm outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
        className,
      )}
      {...props}
    >
      <span className="absolute start-2 flex size-3.5 items-center justify-center">
        <BaseSelect.ItemIndicator className="flex items-center justify-center">
          <CheckIcon className="size-4" />
        </BaseSelect.ItemIndicator>
      </span>
      <BaseSelect.ItemText className="flex flex-col">{children}</BaseSelect.ItemText>
    </BaseSelect.Item>
  ),
)
SelectItem.displayName = "SelectItem"

const SelectSeparator = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseSelect.Separator>
>(({ className, ...props }, ref) => (
  <BaseSelect.Separator
    ref={ref}
    data-slot="select-separator"
    className={cn("-mx-1 my-1 h-px bg-muted", className)}
    {...props}
  />
))
SelectSeparator.displayName = "SelectSeparator"

function SelectScrollUpButton() {
  return (
    <BaseSelect.ScrollUpArrow className="flex cursor-default items-center justify-center py-1">
      <ChevronUpIcon className="size-4" />
    </BaseSelect.ScrollUpArrow>
  )
}

function SelectScrollDownButton() {
  return (
    <BaseSelect.ScrollDownArrow className="flex cursor-default items-center justify-center py-1">
      <ChevronDownIcon className="size-4" />
    </BaseSelect.ScrollDownArrow>
  )
}

export {
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
  SelectScrollUpButton,
  SelectScrollDownButton,
}
