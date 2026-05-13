import React from "react"

interface CustomIconProps extends React.SVGProps<SVGSVGElement> {
  name: string
}

// Map icon names to their SVG paths
const iconPaths: Record<string, string> = {
  // Example: "myicon": "M10 20h30v40H10z"
}

export function CustomIcon({ name, className = "h-4 w-4", ...props }: CustomIconProps) {
  const path = iconPaths[name]

  if (!path) {
    console.warn(`Icon "${name}" not found`)
    return null
  }

  return (
    <svg
      className={className}
      fill="currentColor"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
      {...props}
    >
      <path d={path} />
    </svg>
  )
}
