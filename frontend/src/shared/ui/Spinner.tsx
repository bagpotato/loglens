export interface SpinnerProps {
  label?: string
  className?: string
}

export function Spinner({ label = 'Loading', className = '' }: SpinnerProps) {
  return (
    <span role="status" className={`inline-flex items-center ${className}`}>
      <span
        aria-hidden="true"
        className="size-4 animate-spin rounded-full border-2 border-current/25 border-t-current"
      />
      <span className="sr-only">{label}</span>
    </span>
  )
}
