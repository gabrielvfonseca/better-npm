import { Toast as BaseToast } from "@base-ui/react/toast"
import { XIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const ToastProvider = BaseToast.Provider

const ToastViewport = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseToast.Viewport>
>(({ className, ...props }, ref) => (
  <BaseToast.Viewport
    ref={ref}
    data-slot="toast-viewport"
    className={cn(
      "fixed bottom-0 right-0 z-[100] flex max-h-screen w-full flex-col-reverse gap-2 p-4 sm:max-w-[420px]",
      className,
    )}
    {...props}
  />
))
ToastViewport.displayName = "ToastViewport"

const Toast = React.forwardRef<HTMLDivElement, React.ComponentProps<typeof BaseToast.Root>>(
  ({ className, ...props }, ref) => (
    <BaseToast.Root
      ref={ref}
      data-slot="toast"
      className={cn(
        "group pointer-events-auto relative flex w-full items-center justify-between gap-4 overflow-hidden rounded-md border p-4 pr-6 shadow-lg data-[type=destructive]:border-destructive/50 data-[type=destructive]:text-destructive",
        className,
      )}
      {...props}
    />
  ),
)
Toast.displayName = "Toast"

const ToastAction = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseToast.Action>
>(({ className, ...props }, ref) => (
  <BaseToast.Action
    ref={ref}
    data-slot="toast-action"
    className={cn(
      "inline-flex h-8 shrink-0 items-center justify-center rounded-md border bg-transparent px-3 text-sm font-medium transition-colors hover:bg-secondary focus:outline-none focus:ring-1 focus:ring-ring disabled:pointer-events-none disabled:opacity-50",
      className,
    )}
    {...props}
  />
))
ToastAction.displayName = "ToastAction"

const ToastClose = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseToast.Close>
>(({ className, ...props }, ref) => (
  <BaseToast.Close
    ref={ref}
    data-slot="toast-close"
    className={cn(
      "absolute right-1 top-1 rounded-md p-1 text-foreground/50 opacity-0 transition-opacity hover:text-foreground focus:opacity-100 focus:outline-none focus:ring-1 group-hover:opacity-100",
      className,
    )}
    {...props}
  >
    <XIcon className="size-4" />
  </BaseToast.Close>
))
ToastClose.displayName = "ToastClose"

const ToastTitle = React.forwardRef<
  HTMLHeadingElement,
  React.ComponentProps<typeof BaseToast.Title>
>(({ className, ...props }, ref) => (
  <BaseToast.Title
    ref={ref}
    data-slot="toast-title"
    className={cn("text-sm font-semibold", className)}
    {...props}
  />
))
ToastTitle.displayName = "ToastTitle"

const ToastDescription = React.forwardRef<
  HTMLParagraphElement,
  React.ComponentProps<typeof BaseToast.Description>
>(({ className, ...props }, ref) => (
  <BaseToast.Description
    ref={ref}
    data-slot="toast-description"
    className={cn("text-sm opacity-90", className)}
    {...props}
  />
))
ToastDescription.displayName = "ToastDescription"

function useToast() {
  return BaseToast.useToastManager()
}

function Toaster() {
  return (
    <ToastProvider>
      <ToastList />
    </ToastProvider>
  )
}

function ToastList() {
  const { toasts } = BaseToast.useToastManager()
  return (
    <ToastViewport>
      {toasts.map((toast) => (
        <Toast key={toast.id} toast={toast}>
          <div className="grid gap-1">
            {toast.title ? <ToastTitle>{toast.title}</ToastTitle> : null}
            {toast.description ? <ToastDescription>{toast.description}</ToastDescription> : null}
          </div>
          {toast.actionProps ? <ToastAction {...toast.actionProps} /> : null}
          <ToastClose />
        </Toast>
      ))}
    </ToastViewport>
  )
}

export {
  ToastProvider,
  ToastViewport,
  Toast,
  ToastTitle,
  ToastDescription,
  ToastClose,
  ToastAction,
  Toaster,
  useToast,
}
