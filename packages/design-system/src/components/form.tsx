import { Field } from "@base-ui/react/field"
import * as React from "react"

import { cn } from "../lib/utils"

const Form = React.forwardRef<HTMLFormElement, React.FormHTMLAttributes<HTMLFormElement>>(
  ({ className, ...props }, ref) => (
    <form ref={ref} data-slot="form" className={cn("space-y-4", className)} {...props} />
  ),
)
Form.displayName = "Form"

const FormItem = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <Field.Root
      ref={ref}
      data-slot="form-item"
      className={cn("grid gap-2", className)}
      {...props}
    />
  ),
)
FormItem.displayName = "FormItem"

const FormLabel = React.forwardRef<
  React.ElementRef<typeof Field.Label>,
  React.ComponentProps<typeof Field.Label>
>(({ className, ...props }, ref) => (
  <Field.Label
    ref={ref}
    data-slot="form-label"
    className={cn("text-sm font-medium leading-none", className)}
    {...props}
  />
))
FormLabel.displayName = "FormLabel"

const FormControl = React.forwardRef<
  React.ElementRef<typeof Field.Control>,
  React.ComponentProps<typeof Field.Control>
>(({ className, ...props }, ref) => (
  <Field.Control ref={ref} data-slot="form-control" className={className} {...props} />
))
FormControl.displayName = "FormControl"

const FormDescription = React.forwardRef<
  React.ElementRef<typeof Field.Description>,
  React.ComponentProps<typeof Field.Description>
>(({ className, ...props }, ref) => (
  <Field.Description
    ref={ref}
    data-slot="form-description"
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
FormDescription.displayName = "FormDescription"

const FormMessage = React.forwardRef<
  React.ElementRef<typeof Field.Error>,
  React.ComponentProps<typeof Field.Error>
>(({ className, children, ...props }, ref) => (
  <Field.Error
    ref={ref}
    data-slot="form-message"
    className={cn("text-sm font-medium text-destructive", className)}
    {...props}
  >
    {children}
  </Field.Error>
))
FormMessage.displayName = "FormMessage"

export { Form, FormItem, FormLabel, FormControl, FormDescription, FormMessage }
