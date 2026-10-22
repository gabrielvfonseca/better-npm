import { AlertDialog as BaseAlertDialog } from "@base-ui/react/alert-dialog"
import { XIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const AlertDialog = (props: React.ComponentProps<typeof BaseAlertDialog.Root>) => (
  <BaseAlertDialog.Root data-slot="alert-dialog" {...props} />
)
AlertDialog.displayName = "AlertDialog"

const AlertDialogTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseAlertDialog.Trigger>
>(({ className, ...props }, ref) => (
  <BaseAlertDialog.Trigger
    ref={ref}
    data-slot="alert-dialog-trigger"
    className={className}
    {...props}
  />
))
AlertDialogTrigger.displayName = "AlertDialogTrigger"

const AlertDialogPortal = (props: React.ComponentProps<typeof BaseAlertDialog.Portal>) => (
  <BaseAlertDialog.Portal data-slot="alert-dialog-portal" {...props} />
)
AlertDialogPortal.displayName = "AlertDialogPortal"

const AlertDialogClose = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseAlertDialog.Close>
>(({ className, ...props }, ref) => (
  <BaseAlertDialog.Close
    ref={ref}
    data-slot="alert-dialog-close"
    className={className}
    {...props}
  />
))
AlertDialogClose.displayName = "AlertDialogClose"

const AlertDialogOverlay = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseAlertDialog.Backdrop>
>(({ className, ...props }, ref) => (
  <BaseAlertDialog.Backdrop
    ref={ref}
    data-slot="overlay"
    className={cn("fixed inset-0 z-50 bg-black/50", className)}
    {...props}
  />
))
AlertDialogOverlay.displayName = "AlertDialogOverlay"

const AlertDialogContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseAlertDialog.Popup> & { showCloseButton?: boolean }
>(({ className, children, showCloseButton = true, ...props }, ref) => (
  <AlertDialogPortal>
    <AlertDialogOverlay />
    <BaseAlertDialog.Popup
      ref={ref}
      data-slot="alert-dialog-content"
      className={cn(
        "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border bg-background p-6 shadow-lg rounded-lg duration-200 max-h-[calc(100dvh-2rem)] overflow-y-auto",
        className,
      )}
      {...props}
    >
      {children}
      {showCloseButton && (
        <AlertDialogClose className="absolute right-4 top-4 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2">
          <XIcon className="size-4" />
          <span className="sr-only">Close</span>
        </AlertDialogClose>
      )}
    </BaseAlertDialog.Popup>
  </AlertDialogPortal>
))
AlertDialogContent.displayName = "AlertDialogContent"

function AlertDialogHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="alert-dialog-header"
      className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)}
      {...props}
    />
  )
}

function AlertDialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="alert-dialog-footer"
      className={cn("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2", className)}
      {...props}
    />
  )
}

const AlertDialogTitle = React.forwardRef<
  HTMLHeadingElement,
  React.ComponentProps<typeof BaseAlertDialog.Title>
>(({ className, ...props }, ref) => (
  <BaseAlertDialog.Title
    ref={ref}
    data-slot="alert-dialog-title"
    className={cn("text-lg font-semibold leading-none tracking-tight", className)}
    {...props}
  />
))
AlertDialogTitle.displayName = "AlertDialogTitle"

const AlertDialogDescription = React.forwardRef<
  HTMLParagraphElement,
  React.ComponentProps<typeof BaseAlertDialog.Description>
>(({ className, ...props }, ref) => (
  <BaseAlertDialog.Description
    ref={ref}
    data-slot="alert-dialog-description"
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
AlertDialogDescription.displayName = "AlertDialogDescription"

export {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogPortal,
  AlertDialogClose,
  AlertDialogOverlay,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
}
