# Custom Icons

Store custom SVG icon files in this folder.

## Usage

### Option 1: SVG Files (Recommended)

1. Add your `.svg` files to this folder
2. Import in React:
```tsx
import CustomIcon from "~/assets/icons/my-icon.svg?react"

export function MyComponent() {
  return <CustomIcon className="h-4 w-4" />
}
```

### Option 2: React Components

Create icon components in `app/components/icons/`:
```tsx
// app/components/icons/my-icon.tsx
export function MyIcon({ className = "h-4 w-4" }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z" />
    </svg>
  )
}
```

Then use with shadcn/ui:
```tsx
import { Button } from "~/components/ui/button"
import { MyIcon } from "~/components/icons/my-icon"

export function MyComponent() {
  return <Button><MyIcon className="mr-2 h-4 w-4" /> Click me</Button>
}
```

## Tips

- Use `className="h-4 w-4"` for button icons (standard size)
- Use `fill="currentColor"` to inherit text color
- Test icon visibility in both light and dark modes
