# `@dash/design-system`

React component prop type definitions for the Dash design system.

## Overview

Exports prop type interfaces for design system components: `Button`, `Input`, `Select`, `Card`, `Badge`, `Modal`, `Table<T>`, `Spinner`, `Tabs`, `Tooltip`, `Avatar`, `Dropdown`, `Pagination`, `Toast`, `Skeleton`, `ProgressBar`.

## Usage

```tsx
import type { ButtonProps, ModalProps, TableProps } from "@dash/design-system";

function MyComponent(props: ButtonProps) {
  return <button className={props.variant}>{props.children}</button>;
}
```
