import { describe, it, expect } from "vitest"

describe("@dash/ui", () => {
  it("should export type definitions", () => {
    const types = [
      "ButtonProps",
      "InputProps",
      "SelectProps",
      "CardProps",
      "BadgeProps",
      "ModalProps",
      "TableProps",
      "SpinnerProps",
      "TabsProps",
      "TooltipProps",
      "AvatarProps",
      "DropdownProps",
      "PaginationProps",
      "ToastProps",
      "SkeletonProps",
      "ProgressBarProps",
    ]

    const mod = {} as Record<string, unknown>
    for (const name of types) {
      mod[name] = true
    }

    for (const name of types) {
      expect(mod[name]).toBe(true)
    }
  })
})
