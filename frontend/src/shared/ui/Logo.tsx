export interface LogoProps {
  className?: string
  withText?: boolean
}

export function Logo({ className = 'h-8', withText = false }: LogoProps) {
  return (
    <span className="inline-flex items-center gap-3">
      <svg viewBox="0 0 612 500" className={className} aria-hidden="true">
        <rect width="612" height="500" rx="125" fill="var(--logo-tile)" />
        <rect x="50" y="54" width="372" height="392" rx="92" fill="var(--logo-inner)" />
        <path d="M400 123H541V90L593 141.5L541 193V160H400Z" fill="var(--logo-inner)" />
        <path d="M295 284.5L235 344.5L295 404.5V366H440V323H295Z" fill="var(--logo-tile)" />
        <path d="M101.5 90L204 155L101.5 218V203L180 155L101.5 105.5Z" fill="var(--logo-tile)" />
        <circle cx="215" cy="207.5" r="10.5" fill="var(--logo-tile)" />
      </svg>
      {withText && <span className="text-xl font-bold tracking-tight text-fg">LogLens</span>}
    </span>
  )
}
