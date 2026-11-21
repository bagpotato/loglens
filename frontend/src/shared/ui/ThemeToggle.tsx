import { useTheme } from '@/shared/hooks/useTheme'

export interface ThemeToggleProps {
  className?: string
}

export function ThemeToggle({ className = '' }: ThemeToggleProps) {
  const { theme, toggle } = useTheme()
  const toLight = theme === 'dark'

  return (
    <button
      type="button"
      onClick={toggle}
      aria-label={toLight ? 'Switch to light theme' : 'Switch to dark theme'}
      className={`ui-btn ui-btn--ghost size-10 shrink-0 px-0 ${className}`}
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.8}
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
        className="size-5"
      >
        {toLight ? (
          <>
            <circle cx="12" cy="12" r="4" />
            <path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4L7 17M17 7l1.4-1.4" />
          </>
        ) : (
          <path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z" />
        )}
      </svg>
    </button>
  )
}
