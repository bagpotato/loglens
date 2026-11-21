import type { ComponentType, SVGProps } from 'react'
import { NavLink } from 'react-router'
import { ThemeToggle } from '@/shared/ui/ThemeToggle'
import { LogoutIcon, LogsIcon, OverviewIcon, SparkIcon } from './icons'

interface NavItem {
  to: string
  label: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
  end?: boolean
}

const ITEMS: NavItem[] = [
  { to: '/', label: 'Overview', icon: OverviewIcon, end: true },
  { to: '/logs', label: 'Logs', icon: LogsIcon },
]

export interface SideNavProps {
  username?: string
  chatOpen: boolean
  onToggleChat: () => void
  onLogout: () => void
}

const SIDE_LINK =
  'relative flex h-(--nav-item-height) items-center gap-3 rounded-md px-3 text-base font-semibold transition-colors'

export function SideNav({ username, chatOpen, onToggleChat, onLogout }: SideNavProps) {
  return (
    <nav aria-label="Main" className="flex flex-1 flex-col gap-1 px-2">
      {ITEMS.map(({ to, label, icon: ItemIcon, end }) => (
        <NavLink
          key={to}
          to={to}
          end={end}
          className={({ isActive }) =>
            `${SIDE_LINK} ${isActive ? 'bg-(--nav-active-bg) text-fg before:absolute before:inset-y-2 before:left-0 before:w-0.5 before:rounded-full before:bg-(--nav-active-mark)' : 'text-muted hover:bg-hover hover:text-fg'}`
          }
        >
          <ItemIcon />
          {label}
        </NavLink>
      ))}
      <button
        type="button"
        onClick={onToggleChat}
        aria-pressed={chatOpen}
        className={`${SIDE_LINK} mt-3 border ${chatOpen ? 'border-ai-line bg-ai-bg text-ai' : 'border-line text-ai hover:bg-ai-bg'}`}
      >
        <SparkIcon />
        Ask AI
      </button>
      <div className="mt-auto flex items-center gap-1 border-t border-line py-3 pl-3">
        <span className="min-w-0 flex-1 truncate text-sm text-muted" title={username}>
          {username}
        </span>
        <ThemeToggle />
        <button
          type="button"
          onClick={onLogout}
          aria-label="Log out"
          title="Log out"
          className="ui-btn ui-btn--ghost size-10 px-0"
        >
          <LogoutIcon />
        </button>
      </div>
    </nav>
  )
}

const BOTTOM_ITEMS: NavItem[] = [...ITEMS, { to: '/chat', label: 'Ask AI', icon: SparkIcon }]

export function BottomNav() {
  return (
    <nav
      aria-label="Main"
      className="grid grid-cols-3 border-t border-line bg-(--nav-bg) pb-[env(safe-area-inset-bottom)]"
    >
      {BOTTOM_ITEMS.map(({ to, label, icon: ItemIcon, end }) => (
        <NavLink
          key={to}
          to={to}
          end={end}
          className={({ isActive }) =>
            `relative flex h-14 flex-col items-center justify-center gap-0.5 text-xs font-semibold ${isActive ? 'text-fg before:absolute before:top-0 before:h-0.5 before:w-10 before:bg-(--nav-active-mark)' : 'text-subtle hover:text-fg'}`
          }
        >
          <ItemIcon />
          {label}
        </NavLink>
      ))}
    </nav>
  )
}
