import { Drawer as BaseDrawer } from "@base-ui/react/drawer"
import { XIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const Sheet = (props: React.ComponentProps<typeof BaseDrawer.Root>) => (
  <BaseDrawer.Root data-slot="sheet" {...props} />
)
Sheet.displayName = "Sheet"

const SheetTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseDrawer.Trigger>
>(({ className, ...props }, ref) => (
  <BaseDrawer.Trigger ref={ref} data-slot="sheet-trigger" className={className} {...props} />
))
SheetTrigger.displayName = "SheetTrigger"

const SheetClose = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseDrawer.Close>
>(({ className, ...props }, ref) => (
  <BaseDrawer.Close ref={ref} data-slot="sheet-close" className={className} {...props} />
))
SheetClose.displayName = "SheetClose"

const SheetPortal = (props: React.ComponentProps<typeof BaseDrawer.Portal>) => (
  <BaseDrawer.Portal data-slot="sheet-portal" {...props} />
)
SheetPortal.displayName = "SheetPortal"

const SheetOverlay = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseDrawer.Backdrop>
>(({ className, ...props }, ref) => (
  <BaseDrawer.Backdrop
    ref={ref}
    data-slot="overlay"
    className={cn("fixed inset-0 z-50 bg-black/50", className)}
    {...props}
  />
))
SheetOverlay.displayName = "SheetOverlay"

const SheetContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseDrawer.Popup> & {
    side?: "top" | "right" | "bottom" | "left"
    showCloseButton?: boolean
  }
>(({ className, children, side = "right", showCloseButton = true, ...props }, ref) => (
  <SheetPortal>
    <SheetOverlay />
    <BaseDrawer.Popup
      ref={ref}
      data-slot="sheet-content"
      data-side={side}
      className={cn(
        "fixed z-50 flex flex-col gap-4 bg-background p-6 shadow-lg transition ease-in-out bg-background",
        side === "right" &&
          "inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm data-[swipe-direction=left]:animate-slide-out-right",
        side === "left" && "inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm",
        side === "top" && "inset-x-0 top-0 border-b",
        side === "bottom" && "inset-x-0 bottom-0 border-t",
        className,
      )}
      {...props}
    >
      {children}
      {showCloseButton && (
        <SheetClose className="absolute right-4 top-4 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring">
          <XIcon className="size-4" />
          <span className="sr-only">Close</span>
        </SheetClose>
      )}
    </BaseDrawer.Popup>
  </SheetPortal>
))
SheetContent.displayName = "SheetContent"

function SheetHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="sheet-header"
      className={cn("flex flex-col space-y-2 text-center sm:text-left", className)}
      {...props}
    />
  )
}

function SheetFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="sheet-footer"
      className={cn("mt-auto flex flex-col gap-2 sm:flex-row sm:justify-end", className)}
      {...props}
    />
  )
}

const SheetTitle = React.forwardRef<
  HTMLHeadingElement,
  React.ComponentProps<typeof BaseDrawer.Title>
>(({ className, ...props }, ref) => (
  <BaseDrawer.Title
    ref={ref}
    data-slot="sheet-title"
    className={cn("text-lg font-semibold text-foreground", className)}
    {...props}
  />
))
SheetTitle.displayName = "SheetTitle"

const SheetDescription = React.forwardRef<
  HTMLParagraphElement,
  React.ComponentProps<typeof BaseDrawer.Description>
>(({ className, ...props }, ref) => (
  <BaseDrawer.Description
    ref={ref}
    data-slot="sheet-description"
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
SheetDescription.displayName = "SheetDescription"

export {
  Sheet,
  SheetTrigger,
  SheetClose,
  SheetPortal,
  SheetOverlay,
  SheetContent,
  SheetHeader,
  SheetFooter,
  SheetTitle,
  SheetDescription,
}
