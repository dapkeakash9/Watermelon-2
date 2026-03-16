import { useEffect, useMemo, useRef, useState } from 'react'

type SelectOption = {
  label: string
  value: string
}

type ThemedSelectProps = {
  id?: string
  onChange: (value: string) => void
  options: SelectOption[]
  value: string
}

export function ThemedSelect({ id, onChange, options, value }: ThemedSelectProps) {
  const [open, setOpen] = useState(false)
  const rootRef = useRef<HTMLDivElement | null>(null)

  const selectedOption = useMemo(
    () => options.find((option) => option.value === value) ?? options[0],
    [options, value],
  )

  useEffect(() => {
    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false)
      }
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setOpen(false)
      }
    }

    window.addEventListener('mousedown', handlePointerDown)
    window.addEventListener('keydown', handleKeyDown)

    return () => {
      window.removeEventListener('mousedown', handlePointerDown)
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [])

  return (
    <div className="themed-select" ref={rootRef}>
      <button
        aria-controls={id ? `${id}-listbox` : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        className="themed-select-trigger"
        id={id}
        onClick={() => setOpen((current) => !current)}
        type="button"
      >
        <span>{selectedOption?.label ?? value}</span>
        <span className="themed-select-caret" aria-hidden="true">
          ▾
        </span>
      </button>

      {open ? (
        <div
          className="themed-select-menu"
          id={id ? `${id}-listbox` : undefined}
          role="listbox"
        >
          {options.map((option) => (
            <button
              key={option.value}
              aria-selected={option.value === value}
              className={option.value === value ? 'themed-select-option active' : 'themed-select-option'}
              onClick={() => {
                onChange(option.value)
                setOpen(false)
              }}
              role="option"
              type="button"
            >
              {option.label}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  )
}
