import { Dialog as BaseDialog } from "@base-ui/react/dialog"
import { XIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const Dialog = (props: React.ComponentProps<typeof BaseDialog.Root>) => (
  <BaseDialog.Root data-slot="dialog" {...props} />
)
Dialog.displayName = "Dialog"

const DialogTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseDialog.Trigger>
>(({ className, ...props }, ref) => (
  <BaseDialog.Trigger ref={ref} data-slot="dialog-trigger" className={className} {...props} />
))
DialogTrigger.displayName = "DialogTrigger"

const DialogPortal = (props: React.ComponentProps<typeof BaseDialog.Portal>) => (
  <BaseDialog.Portal data-slot="dialog-portal" {...props} />
)
DialogPortal.displayName = "DialogPortal"

const DialogClose = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseDialog.Close>
>(({ className, ...props }, ref) => (
  <BaseDialog.Close ref={ref} data-slot="dialog-close" className={className} {...props} />
))
DialogClose.displayName = "DialogClose"

const DialogOverlay = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseDialog.Backdrop>
>(({ className, ...props }, ref) => (
  <BaseDialog.Backdrop
    ref={ref}
    data-slot="overlay"
    className={cn("fixed inset-0 z-50 bg-black/50", className)}
    {...props}
  />
))
DialogOverlay.displayName = "DialogOverlay"

const DialogContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseDialog.Popup> & { showCloseButton?: boolean }
>(({ className, children, showCloseButton = true, ...props }, ref) => (
  <DialogPortal>
    <DialogOverlay />
    <BaseDialog.Popup
      ref={ref}
      data-slot="dialog-content"
      className={cn(
        "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border bg-background p-6 shadow-lg rounded-lg duration-200 max-h-[calc(100dvh-2rem)] overflow-y-auto",
        className,
      )}
      {...props}
    >
      {children}
      {showCloseButton && (
        <DialogClose className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none">
          <XIcon className="size-4" />
          <span className="sr-only">Close</span>
        </DialogClose>
      )}
    </BaseDialog.Popup>
  </DialogPortal>
))
DialogContent.displayName = "DialogContent"

function DialogHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="dialog-header"
      className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)}
      {...props}
    />
  )
}

function DialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="dialog-footer"
      className={cn("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2", className)}
      {...props}
    />
  )
}

const DialogTitle = React.forwardRef<
  HTMLHeadingElement,
  React.ComponentProps<typeof BaseDialog.Title>
>(({ className, ...props }, ref) => (
  <BaseDialog.Title
    ref={ref}
    data-slot="dialog-title"
    className={cn("text-lg font-semibold leading-none tracking-tight", className)}
    {...props}
  />
))
DialogTitle.displayName = "DialogTitle"

const DialogDescription = React.forwardRef<
  HTMLParagraphElement,
  React.ComponentProps<typeof BaseDialog.Description>
>(({ className, ...props }, ref) => (
  <BaseDialog.Description
    ref={ref}
    data-slot="dialog-description"
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
DialogDescription.displayName = "DialogDescription"

export {
  Dialog,
  DialogTrigger,
  DialogPortal,
  DialogClose,
  DialogOverlay,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
}
