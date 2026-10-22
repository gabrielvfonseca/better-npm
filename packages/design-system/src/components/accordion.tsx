import { Accordion as BaseAccordion } from "@base-ui/react/accordion"
import { ChevronDownIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

const Accordion = React.forwardRef<HTMLDivElement, React.ComponentProps<typeof BaseAccordion.Root>>(
  ({ className, ...props }, ref) => (
    <BaseAccordion.Root
      ref={ref}
      data-slot="accordion"
      className={cn("w-full", className)}
      {...props}
    />
  ),
)
Accordion.displayName = "Accordion"

const AccordionItem = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseAccordion.Item>
>(({ className, ...props }, ref) => (
  <BaseAccordion.Item
    ref={ref}
    data-slot="accordion-item"
    className={cn("border-b last:border-b-0", className)}
    {...props}
  />
))
AccordionItem.displayName = "AccordionItem"

const AccordionTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<typeof BaseAccordion.Trigger>
>(({ className, children, ...props }, ref) => (
  <BaseAccordion.Header className="flex">
    <BaseAccordion.Trigger
      ref={ref}
      data-slot="accordion-trigger"
      className={cn(
        "flex flex-1 items-center justify-between py-4 text-left text-sm font-medium transition-all hover:underline focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring [&[data-panel-open]>svg]:rotate-180",
        className,
      )}
      {...props}
    >
      {children}
      <ChevronDownIcon className="size-4 shrink-0 text-muted-foreground transition-transform duration-200" />
    </BaseAccordion.Trigger>
  </BaseAccordion.Header>
))
AccordionTrigger.displayName = "AccordionTrigger"

const AccordionContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseAccordion.Panel>
>(({ className, children, ...props }, ref) => (
  <BaseAccordion.Panel
    ref={ref}
    data-slot="accordion-content"
    className="overflow-hidden text-sm data-[starting-style]:animate-accordion-up data-[ending-style]:animate-accordion-down"
    {...props}
  >
    <div className={cn("pb-4 pt-0", className)}>{children}</div>
  </BaseAccordion.Panel>
))
AccordionContent.displayName = "AccordionContent"

export { Accordion, AccordionItem, AccordionTrigger, AccordionContent }
