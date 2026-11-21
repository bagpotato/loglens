import { useCallback, useState } from 'react'
import { applyTheme, readTheme, storeTheme, type Theme } from '@/shared/theme'

export function useTheme(): { theme: Theme; toggle: () => void } {
  const [theme, setTheme] = useState<Theme>(readTheme)

  const toggle = useCallback(() => {
    const next: Theme = theme === 'dark' ? 'light' : 'dark'
    applyTheme(next)
    storeTheme(next)
    setTheme(next)
  }, [theme])

  return { theme, toggle }
}
