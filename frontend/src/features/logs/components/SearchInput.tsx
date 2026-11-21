import { useEffect, useRef, useState } from 'react'

const DEBOUNCE_MS = 300

export interface SearchInputProps {
  value: string
  onChange: (value: string) => void
}

export function SearchInput({ value, onChange }: SearchInputProps) {
  const [text, setText] = useState(value)
  const [lastValue, setLastValue] = useState(value)
  const onChangeRef = useRef(onChange)

  if (value !== lastValue) {
    setLastValue(value)
    setText(value)
  }

  useEffect(() => {
    onChangeRef.current = onChange
  }, [onChange])

  useEffect(() => {
    if (text === value) return
    const timer = setTimeout(() => onChangeRef.current(text), DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [text, value])

  return (
    <input
      type="search"
      value={text}
      onChange={(event) => setText(event.target.value)}
      placeholder="Search messages…"
      aria-label="Search messages"
      className="ui-input h-11 max-lg:text-[16px] lg:h-(--input-height)"
    />
  )
}
