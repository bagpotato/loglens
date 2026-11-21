export interface LogoProps {
  className?: string
  withText?: boolean
}

export function Logo({ className = 'h-8', withText = false }: LogoProps) {
  return (
    <span className="inline-flex items-center gap-3">
      <svg viewBox="768 552 675 553" className={className} aria-hidden="true">
        <rect x="768" y="552" width="675" height="553" rx="138" fill="var(--logo-tile)" />
        <rect x="821" y="612" width="416" height="434" rx="104" fill="var(--logo-inner)" />
        <path d="M1220 689H1366V652L1423 710L1366 767V729H1220Z" fill="var(--logo-inner)" />
        <path d="M1094 866L1027 933L1094 1001V957H1244V910H1094Z" fill="var(--logo-tile)" />
        <path d="M884 660L981 724L884 786" fill="none" stroke="var(--logo-tile)" strokeWidth="17" />
        <circle cx="1005" cy="783" r="12" fill="var(--logo-tile)" />
      </svg>
      {withText && <span className="text-xl font-bold tracking-tight text-fg">LogLens</span>}
    </span>
  )
}
