import { useState } from 'react'
import { Outlet } from 'react-router'
import { ChatPanel } from '@/features/ai'
import { useAuth } from '@/features/auth'
import { useIsDesktop } from '@/shared/hooks/useMediaQuery'
import { Logo } from '@/shared/ui/Logo'
import { ThemeToggle } from '@/shared/ui/ThemeToggle'
import { LogoutIcon } from './icons'
import { BottomNav, SideNav } from './Navigation'

export function Layout() {
  const { user, logout } = useAuth()
  const isDesktop = useIsDesktop()
  const [chatOpen, setChatOpen] = useState(false)
  const showChat = isDesktop && chatOpen

  return (
    <div className="flex h-dvh overflow-hidden bg-bg text-fg">
      <aside className="hidden w-(--nav-width) shrink-0 flex-col border-r border-line bg-(--nav-bg) lg:flex">
        <div className="px-4 pt-5 pb-4">
          <Logo className="h-7" withText />
        </div>
        <SideNav
          username={user?.username}
          chatOpen={chatOpen}
          onToggleChat={() => setChatOpen((open) => !open)}
          onLogout={logout}
        />
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 shrink-0 items-center justify-between border-b border-line bg-(--nav-bg) pr-2 pl-4 pt-[env(safe-area-inset-top)] box-content lg:hidden">
          <Logo className="h-6" withText />
          <div className="flex items-center">
            <ThemeToggle />
            <button
              type="button"
              onClick={logout}
              aria-label="Log out"
              className="ui-btn ui-btn--ghost size-10 px-0"
            >
              <LogoutIcon />
            </button>
          </div>
        </header>
        <main className="flex min-h-0 flex-1 flex-col overflow-y-auto">
          <Outlet />
        </main>
        <div className="shrink-0 lg:hidden">
          <BottomNav />
        </div>
      </div>

      {showChat && (
        <aside
          aria-label="AI assistant"
          className="flex w-(--chat-panel-width) shrink-0 flex-col border-l border-line bg-surface"
        >
          <ChatPanel onClose={() => setChatOpen(false)} />
        </aside>
      )}
    </div>
  )
}
