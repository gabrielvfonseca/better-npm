import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react"
import * as React from "react"

import { cn } from "../lib/utils"

export interface PaginationProps {
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  siblingCount?: number
  className?: string
}

function getPageRange(
  page: number,
  totalPages: number,
  siblingCount: number,
): (number | "ellipsis")[] {
  const totalNumbers = siblingCount * 2 + 5
  if (totalPages <= totalNumbers) {
    return Array.from({ length: totalPages }, (_, i) => i + 1)
  }

  const leftSibling = Math.max(page - siblingCount, 1)
  const rightSibling = Math.min(page + siblingCount, totalPages)
  const showLeftEllipsis = leftSibling > 2
  const showRightEllipsis = rightSibling < totalPages - 1

  const range: (number | "ellipsis")[] = [1]
  if (showLeftEllipsis) range.push("ellipsis")
  for (let i = leftSibling; i <= rightSibling; i++) range.push(i)
  if (showRightEllipsis) range.push("ellipsis")
  range.push(totalPages)
  return range
}

const Pagination = React.forwardRef<HTMLElement, PaginationProps>(
  ({ page, totalPages, onPageChange, siblingCount = 1, className }, ref) => {
    const pages = React.useMemo(
      () => getPageRange(page, totalPages, siblingCount),
      [page, totalPages, siblingCount],
    )

    const baseItem =
      "inline-flex size-9 items-center justify-center rounded-md border text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"

    let ellipsisKey = 0

    return (
      <nav
        ref={ref}
        data-slot="pagination"
        aria-label="pagination"
        className={cn("mx-auto flex w-full justify-center", className)}
      >
        <ul className="flex flex-row items-center gap-1">
          <li>
            <button
              type="button"
              aria-label="Go to previous page"
              className={cn(baseItem, "hover:bg-accent hover:text-accent-foreground")}
              onClick={() => onPageChange(Math.max(1, page - 1))}
              disabled={page <= 1}
            >
              <ChevronLeftIcon className="size-4" />
            </button>
          </li>
          {pages.map((p) =>
            p === "ellipsis" ? (
              <li key={`ellipsis-${ellipsisKey++}`} className="px-2 text-sm">
                &#8230;
              </li>
            ) : (
              <li key={p}>
                <button
                  type="button"
                  aria-label={`Go to page ${p}`}
                  aria-current={p === page ? "page" : undefined}
                  className={cn(
                    baseItem,
                    p === page
                      ? "border-primary bg-primary text-primary-foreground"
                      : "border-input bg-background hover:bg-accent hover:text-accent-foreground",
                  )}
                  onClick={() => onPageChange(p)}
                >
                  {p}
                </button>
              </li>
            ),
          )}
          <li>
            <button
              type="button"
              aria-label="Go to next page"
              className={cn(baseItem, "hover:bg-accent hover:text-accent-foreground")}
              onClick={() => onPageChange(Math.min(totalPages, page + 1))}
              disabled={page >= totalPages}
            >
              <ChevronRightIcon className="size-4" />
            </button>
          </li>
        </ul>
      </nav>
    )
  },
)
Pagination.displayName = "Pagination"

export { Pagination }
