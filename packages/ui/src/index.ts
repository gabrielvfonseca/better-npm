export interface ButtonProps {
  variant?: "primary" | "secondary" | "ghost" | "danger"
  size?: "sm" | "md" | "lg"
  loading?: boolean
  disabled?: boolean
  fullWidth?: boolean
  type?: "button" | "submit" | "reset"
  children: React.ReactNode
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void
  className?: string
}

export interface InputProps {
  label?: string
  error?: string
  hint?: string
  size?: "sm" | "md" | "lg"
  fullWidth?: boolean
  disabled?: boolean
  type?: "text" | "email" | "password" | "number" | "search" | "url"
  placeholder?: string
  value?: string
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void
  className?: string
}

export interface SelectProps {
  label?: string
  error?: string
  options: Array<{ value: string; label: string }>
  value?: string
  onChange?: (value: string) => void
  placeholder?: string
  disabled?: boolean
  fullWidth?: boolean
  className?: string
}

export interface CardProps {
  title?: string
  subtitle?: string
  padding?: "none" | "sm" | "md" | "lg"
  variant?: "elevated" | "outlined" | "flat"
  children: React.ReactNode
  className?: string
  onClick?: () => void
}

export interface BadgeProps {
  variant?: "default" | "success" | "warning" | "error" | "info"
  size?: "sm" | "md"
  children: React.ReactNode
  className?: string
}

export interface ModalProps {
  open: boolean
  onClose: () => void
  title?: string
  size?: "sm" | "md" | "lg" | "full"
  children: React.ReactNode
  closeOnOverlay?: boolean
  showCloseButton?: boolean
}

export interface TableProps<T> {
  columns: Array<{ key: keyof T; header: string; render?: (item: T) => React.ReactNode }>
  data: T[]
  loading?: boolean
  emptyMessage?: string
  onRowClick?: (item: T) => void
  className?: string
}

export interface SpinnerProps {
  size?: "sm" | "md" | "lg"
  color?: string
  className?: string
}

export interface TabsProps {
  tabs: Array<{ id: string; label: string; badge?: number }>
  activeTab?: string
  onChange: (tabId: string) => void
  className?: string
}

export interface TooltipProps {
  content: string
  position?: "top" | "bottom" | "left" | "right"
  delay?: number
  children: React.ReactNode
}

export interface AvatarProps {
  src?: string
  alt?: string
  initials?: string
  size?: "sm" | "md" | "lg" | "xl"
  status?: "online" | "offline" | "away"
  className?: string
}

export interface DropdownProps {
  trigger: React.ReactNode
  items: Array<{ label: string; value: string; icon?: React.ReactNode; disabled?: boolean; divider?: boolean }>
  onSelect: (value: string) => void
  align?: "start" | "end"
  className?: string
}

export interface PaginationProps {
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  siblingCount?: number
}

export interface ToastProps {
  message: string
  type?: "success" | "error" | "warning" | "info"
  duration?: number
  onDismiss?: () => void
  action?: { label: string; onClick: () => void }
}

export interface SkeletonProps {
  variant?: "text" | "circular" | "rectangular"
  width?: string | number
  height?: string | number
  count?: number
  className?: string
}

export interface ProgressBarProps {
  value: number
  max?: number
  variant?: "determinate" | "indeterminate"
  color?: "primary" | "success" | "warning" | "error"
  size?: "sm" | "md" | "lg"
  showLabel?: boolean
  className?: string
}
