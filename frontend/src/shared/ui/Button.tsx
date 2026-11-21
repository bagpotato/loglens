import type { ButtonHTMLAttributes } from 'react'

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'
export type ButtonSize = 'sm' | 'md'

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  size?: ButtonSize
}

export function Button({
  variant = 'primary',
  size = 'md',
  type = 'button',
  className = '',
  ...props
}: ButtonProps) {
  const sizeClass = size === 'sm' ? 'ui-btn--sm' : ''
  return (
    <button
      type={type}
      className={`ui-btn ui-btn--${variant} ${sizeClass} ${className}`}
      {...props}
    />
  )
}
