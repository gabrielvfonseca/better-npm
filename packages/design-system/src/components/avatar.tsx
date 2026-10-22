import { Avatar as BaseAvatar } from "@base-ui/react/avatar"
import * as React from "react"

import { cn } from "../lib/utils"

const Avatar = React.forwardRef<HTMLDivElement, React.ComponentProps<typeof BaseAvatar.Root>>(
  ({ className, ...props }, ref) => (
    <BaseAvatar.Root
      ref={ref}
      data-slot="avatar"
      className={cn("relative flex size-8 shrink-0 overflow-hidden rounded-full", className)}
      {...props}
    />
  ),
)
Avatar.displayName = "Avatar"

const AvatarImage = React.forwardRef<
  HTMLImageElement,
  React.ComponentProps<typeof BaseAvatar.Image>
>(({ className, ...props }, ref) => (
  <BaseAvatar.Image
    ref={ref}
    data-slot="avatar-image"
    className={cn("aspect-square size-full", className)}
    {...props}
  />
))
AvatarImage.displayName = "AvatarImage"

const AvatarFallback = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof BaseAvatar.Fallback>
>(({ className, ...props }, ref) => (
  <BaseAvatar.Fallback
    ref={ref}
    data-slot="avatar-fallback"
    className={cn(
      "flex size-full items-center justify-center rounded-full bg-muted text-sm font-medium",
      className,
    )}
    {...props}
  />
))
AvatarFallback.displayName = "AvatarFallback"

export { Avatar, AvatarImage, AvatarFallback }
