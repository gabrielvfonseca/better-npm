import { Menu as BaseMenu } from "@base-ui/react/menu"
import { CheckIcon, ChevronRightIcon, CircleIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const DropdownMenu = (props: React.ComponentProps<typeof BaseMenu.Root>) => (
  <BaseMenu.Root data-slot="dropdown-menu" {...props} />
)
DropdownMenu.displayName = "DropdownMenu"

const DropdownMenuTrigger = React.forwardRef<
  React.ElementRef<typeof BaseMenu.Trigger>,
  React.ComponentProps<typeof BaseMenu.Trigger>
>(({ className, ...props }, ref) => (
  <BaseMenu.Trigger
    ref={ref as React.Ref<typeof BaseMenu.Trigger>}
    data-slot="dropdown-menu-trigger"
    className={className}
    {...props}
  />
))
DropdownMenuTrigger.displayName = "DropdownMenuTrigger"

const DropdownMenuGroup = React.forwardRef<
  React.ElementRef<typeof BaseMenu.Group>,
  React.ComponentProps<typeof BaseMenu.Group>
>(({ className, ...props }, ref) => (
  <BaseMenu.Group
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-group"
    className={className}
    {...props}
  />
))
DropdownMenuGroup.displayName = "DropdownMenuGroup"

const DropdownMenuPortal = (props: React.ComponentProps<typeof BaseMenu.Portal>) => (
  <BaseMenu.Portal data-slot="dropdown-menu-portal" {...props} />
)
DropdownMenuPortal.displayName = "DropdownMenuPortal"

const DropdownMenuSub = (props: React.ComponentProps<typeof BaseMenu.SubmenuRoot>) => (
  <BaseMenu.SubmenuRoot data-slot="dropdown-menu-sub" {...props} />
)
DropdownMenuSub.displayName = "DropdownMenuSub"

const DropdownMenuRadioGroup = React.forwardRef<
  React.ElementRef<typeof BaseMenu.RadioGroup>,
  React.ComponentProps<typeof BaseMenu.RadioGroup>
>(({ className, ...props }, ref) => (
  <BaseMenu.RadioGroup
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-radio-group"
    className={className}
    {...props}
  />
))
DropdownMenuRadioGroup.displayName = "DropdownMenuRadioGroup"

const DropdownMenuSubTrigger = React.forwardRef<
  React.ElementRef<typeof BaseMenu.SubmenuTrigger>,
  React.ComponentProps<typeof BaseMenu.SubmenuTrigger>
>(({ className, children, ...props }, ref) => (
  <BaseMenu.SubmenuTrigger
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-sub-trigger"
    className={cn(
      "flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none focus:bg-accent data-[state=open]:bg-accent",
      className,
    )}
    {...props}
  >
    {children}
    <ChevronRightIcon className="ms-auto size-4" />
  </BaseMenu.SubmenuTrigger>
))
DropdownMenuSubTrigger.displayName = "DropdownMenuSubTrigger"

const DropdownMenuSubContent = React.forwardRef<
  React.ElementRef<typeof BaseMenu.Popup>,
  React.ComponentProps<typeof BaseMenu.Positioner>
>(({ className, children, ...props }, ref) => (
  <DropdownMenuPortal>
    <BaseMenu.Positioner
      data-slot="dropdown-menu-sub-positioner"
      sideOffset={4}
      alignOffset={-4}
      className="z-50"
      {...props}
    >
      <BaseMenu.Popup
        ref={ref as React.Ref<typeof BaseDialog.Trigger>}
        data-slot="dropdown-menu-sub-content"
        className={cn(
          "z-50 min-w-32 overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-lg",
          className,
        )}
      >
        {children}
      </BaseMenu.Popup>
    </BaseMenu.Positioner>
  </DropdownMenuPortal>
))
DropdownMenuSubContent.displayName = "DropdownMenuSubContent"

const DropdownMenuContent = React.forwardRef<
  React.ElementRef<typeof BaseMenu.Popup>,
  React.ComponentProps<typeof BaseMenu.Positioner> & { portal?: boolean }
>(({ className, children, portal = true, sideOffset = 4, align = "end", ...props }, ref) => {
  const positioner = (
    <BaseMenu.Positioner
      data-slot="dropdown-menu-positioner"
      sideOffset={sideOffset}
      align={align}
      className="z-50"
      {...props}
    >
      <BaseMenu.Popup
        ref={ref as React.Ref<typeof BaseDialog.Trigger>}
        data-slot="dropdown-menu-content"
        className={cn(
          "z-50 min-w-32 overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md",
          className,
        )}
      >
        {children}
      </BaseMenu.Popup>
    </BaseMenu.Positioner>
  )
  return portal ? <DropdownMenuPortal>{positioner}</DropdownMenuPortal> : positioner
})
DropdownMenuContent.displayName = "DropdownMenuContent"

const DropdownMenuItem = React.forwardRef<
  React.ElementRef<typeof BaseMenu.Item>,
  React.ComponentProps<typeof BaseMenu.Item> & { inset?: boolean }
>(({ className, inset, ...props }, ref) => (
  <BaseMenu.Item
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-item"
    className={cn(
      "relative flex cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0",
      inset && "ps-8",
      className,
    )}
    {...props}
  />
))
DropdownMenuItem.displayName = "DropdownMenuItem"

const DropdownMenuCheckboxItem = React.forwardRef<
  React.ElementRef<typeof BaseMenu.CheckboxItem>,
  React.ComponentProps<typeof BaseMenu.CheckboxItem>
>(({ className, children, checked, ...props }, ref) => (
  <BaseMenu.CheckboxItem
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-checkbox-item"
    className={cn(
      "relative flex cursor-default select-none items-center rounded-sm py-1.5 pe-2 ps-8 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className,
    )}
    checked={checked}
    {...props}
  >
    <span className="absolute start-2 flex size-3.5 items-center justify-center">
      <BaseMenu.CheckboxItemIndicator className="flex items-center justify-center">
        <CheckIcon className="size-4" />
      </BaseMenu.CheckboxItemIndicator>
    </span>
    {children}
  </BaseMenu.CheckboxItem>
))
DropdownMenuCheckboxItem.displayName = "DropdownMenuCheckboxItem"

const DropdownMenuRadioItem = React.forwardRef<
  React.ElementRef<typeof BaseMenu.RadioItem>,
  React.ComponentProps<typeof BaseMenu.RadioItem>
>(({ className, children, ...props }, ref) => (
  <BaseMenu.RadioItem
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-radio-item"
    className={cn(
      "relative flex cursor-default select-none items-center rounded-sm py-1.5 pe-2 ps-8 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className,
    )}
    {...props}
  >
    <span className="absolute start-2 flex size-3.5 items-center justify-center">
      <BaseMenu.RadioItemIndicator className="flex items-center justify-center">
        <CircleIcon className="size-2 fill-current" />
      </BaseMenu.RadioItemIndicator>
    </span>
    {children}
  </BaseMenu.RadioItem>
))
DropdownMenuRadioItem.displayName = "DropdownMenuRadioItem"

const DropdownMenuLabel = React.forwardRef<
  React.ElementRef<typeof BaseMenu.GroupLabel>,
  React.ComponentProps<typeof BaseMenu.GroupLabel> & { inset?: boolean }
>(({ className, inset, ...props }, ref) => (
  <BaseMenu.GroupLabel
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-label"
    className={cn("px-2 py-1.5 text-sm font-semibold", inset && "ps-8", className)}
    {...props}
  />
))
DropdownMenuLabel.displayName = "DropdownMenuLabel"

const DropdownMenuSeparator = React.forwardRef<
  React.ElementRef<typeof BaseMenu.Separator>,
  React.ComponentProps<typeof BaseMenu.Separator>
>(({ className, ...props }, ref) => (
  <BaseMenu.Separator
    ref={ref as React.Ref<typeof BaseDialog.Trigger>}
    data-slot="dropdown-menu-separator"
    className={cn("-mx-1 my-1 h-px bg-muted", className)}
    {...props}
  />
))
DropdownMenuSeparator.displayName = "DropdownMenuSeparator"

function DropdownMenuShortcut({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      data-slot="dropdown-menu-shortcut"
      className={cn("ms-auto text-xs tracking-widest opacity-60", className)}
      {...props}
    />
  )
}

export {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuGroup,
  DropdownMenuPortal,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuRadioGroup,
}
