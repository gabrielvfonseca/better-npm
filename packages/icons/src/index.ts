export type IconName =
  | "search"
  | "chevron-down"
  | "chevron-up"
  | "chevron-left"
  | "chevron-right"
  | "x"
  | "check"
  | "menu"
  | "home"
  | "user"
  | "settings"
  | "bell"
  | "mail"
  | "star"
  | "heart"
  | "share"
  | "download"
  | "upload"
  | "trash"
  | "edit"
  | "plus"
  | "minus"
  | "info"
  | "warning"
  | "alert-circle"
  | "check-circle"
  | "x-circle"
  | "copy"
  | "external-link"
  | "package"
  | "globe"
  | "lock"
  | "unlock"
  | "clock"
  | "calendar"
  | "filter"
  | "sort"
  | "more-horizontal"
  | "more-vertical"
  | "refresh-cw"
  | "loader"
  | "file"
  | "folder"
  | "tag"
  | "terminal"
  | "github"
  | "npm"

export interface IconProps {
  name: IconName
  size?: number
  className?: string
}

export const ICON_NAMES: IconName[] = [
  "search",
  "chevron-down",
  "chevron-up",
  "chevron-left",
  "chevron-right",
  "x",
  "check",
  "menu",
  "home",
  "user",
  "settings",
  "bell",
  "mail",
  "star",
  "heart",
  "share",
  "download",
  "upload",
  "trash",
  "edit",
  "plus",
  "minus",
  "info",
  "warning",
  "alert-circle",
  "check-circle",
  "x-circle",
  "copy",
  "external-link",
  "package",
  "globe",
  "lock",
  "unlock",
  "clock",
  "calendar",
  "filter",
  "sort",
  "more-horizontal",
  "more-vertical",
  "refresh-cw",
  "loader",
  "file",
  "folder",
  "tag",
  "terminal",
  "github",
  "npm",
]
