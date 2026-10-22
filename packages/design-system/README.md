# `@better-npm/design-system`

A comprehensive set of shadcn/ui-style React primitives for the Bnpm design system, built on top of [Base UI](https://base-ui.com) (`@base-ui/react`) instead of Radix UI.

All components are unstyled, accessible, and composable. They ship as TypeScript source and follow shadcn conventions (`cn` utility, `class-variance-authority` variants, `data-slot` hooks, and Base UI's `render` composition model).

## Installation

```bash
bun add @better-npm/design-system @base-ui/react
```

Peer dependencies: `react` and `react-dom` (^19).

## Usage

```tsx
import { Button, Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@better-npm/design-system"

export function Example() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">Open</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Are you sure?</DialogTitle>
          <DialogDescription>This action cannot be undone.</DialogDescription>
        </DialogHeader>
      </DialogContent>
    </Dialog>
  )
}
```

## Theming

The components reference semantic design tokens (`bg-background`, `text-foreground`, `border-input`, `bg-primary`, …) that must be provided by your app:

1. Import the token stylesheet once at your app root:

   ```ts
   import "@better-npm/design-system/styles.css"
   ```

2. Extend your Tailwind theme with the bundled preset (maps the CSS variables to Tailwind colors and exposes the `dark` variant):

   ```ts
   import designSystem from "@better-npm/design-system/tailwind-preset"

   export default {
     presets: [designSystem],
     content: ["./src/**/*.{ts,tsx}", "./node_modules/@better-npm/design-system/src/**/*.{ts,tsx}"],
   }
   ```

The `components.json` in this package records the shadcn configuration (`"base": "base"`) so the official shadcn CLI can keep these primitives in sync.

## Components

**Primitives:** `Button`, `Card` (+ `CardHeader`/`CardTitle`/`CardDescription`/`CardContent`/`CardFooter`), `Badge`, `Input`, `Textarea`, `Label`, `Alert` (+ `AlertTitle`/`AlertDescription`), `Skeleton`, `Spinner`, `Separator`, `Avatar` (+ `AvatarImage`/`AvatarFallback`).

**Forms:** `Checkbox`, `Switch`, `RadioGroup` (+ `RadioGroupItem`), `Select` (+ `SelectTrigger`/`SelectContent`/`SelectItem`/`SelectValue`/`SelectLabel`/`SelectGroup`/`SelectSeparator`), `Slider`, `Progress`, `Accordion`, `Form` (+ `FormItem`/`FormLabel`/`FormControl`/`FormDescription`/`FormMessage`).

**Overlays:** `Dialog`, `AlertDialog`, `Sheet` (Drawer), `Popover`, `Tooltip` (+ `TooltipProvider`), `DropdownMenu`.

**Display & navigation:** `Tabs`, `Toast` (+ `Toaster`, `useToast`, `ToastProvider`), `Pagination`, `Table` (+ `TableHeader`/`TableBody`/`TableFooter`/`TableHead`/`TableRow`/`TableCell`/`TableCaption`).

## Utilities

- `cn(...classes)` — `clsx` + `tailwind-merge` helper.
- `buttonVariants` / `badgeVariants` — `cva` variant resolvers for composition via Base UI's `render` prop.
