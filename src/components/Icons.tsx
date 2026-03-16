import type { ReactNode, SVGProps } from 'react'

type IconProps = SVGProps<SVGSVGElement> & {
  title?: string
}

function BaseIcon({ children, title, ...props }: IconProps & { children: ReactNode }) {
  return (
    <svg
      aria-hidden={title ? undefined : true}
      fill="none"
      height="20"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.75"
      viewBox="0 0 24 24"
      width="20"
      {...props}
    >
      {title ? <title>{title}</title> : null}
      {children}
    </svg>
  )
}

export function HomeIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M4 10.5 12 4l8 6.5" />
      <path d="M6.5 9.5V20h11V9.5" />
      <path d="M10 20v-5.5h4V20" />
    </BaseIcon>
  )
}

export function DownloadIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M12 4v10" />
      <path d="m8.5 11.5 3.5 3.5 3.5-3.5" />
      <path d="M5 19h14" />
    </BaseIcon>
  )
}

export function SparkIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="m12 3 1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4L12 3Z" />
      <path d="m18.5 15 .7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7.7-2.3Z" />
    </BaseIcon>
  )
}

export function SettingsIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M6 7.5h12" />
      <path d="M6 12h12" />
      <path d="M6 16.5h12" />
      <circle cx="9" cy="7.5" fill="currentColor" r="1.8" stroke="none" />
      <circle cx="15" cy="12" fill="currentColor" r="1.8" stroke="none" />
      <circle cx="11" cy="16.5" fill="currentColor" r="1.8" stroke="none" />
    </BaseIcon>
  )
}

export function PlayIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M8 6.5v11l9-5.5-9-5.5Z" fill="currentColor" stroke="none" />
    </BaseIcon>
  )
}

export function FolderIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M3.5 8.5h6l2 2h9v7.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2V8.5Z" />
      <path d="M3.5 8.5V7a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v1.5" />
    </BaseIcon>
  )
}

export function CloseIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="m7 7 10 10" />
      <path d="M17 7 7 17" />
    </BaseIcon>
  )
}

export function CheckIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="m6.5 12.5 3.5 3.5 7.5-8" />
    </BaseIcon>
  )
}

export function WatermelonIcon(props: IconProps) {
  return (
    <svg
      aria-hidden={props.title ? undefined : true}
      fill="none"
      viewBox="0 0 64 64"
      {...props}
    >
      {props.title ? <title>{props.title}</title> : null}
      <path
        d="M10 36c2.2 11.8 12.6 20 22 20s19.8-8.2 22-20c-13.6-5.7-30.4-5.7-44 0Z"
        fill="#FF5F6D"
      />
      <path
        d="M8 34.5c15.3-7.2 32.7-7.2 48 0"
        stroke="#13241C"
        strokeLinecap="round"
        strokeWidth="3"
      />
      <path
        d="M12 39c2.7 11 11.7 18 20 18s17.3-7 20-18"
        stroke="#57D3A5"
        strokeLinecap="round"
        strokeWidth="6"
      />
      <path d="M24 43.5c0 1.9-1 3.5-2.2 3.5s-2.2-1.6-2.2-3.5S20.6 40 21.8 40s2.2 1.6 2.2 3.5Z" fill="#13241C" />
      <path d="M34.2 47c0 1.9-1 3.5-2.2 3.5s-2.2-1.6-2.2-3.5 1-3.5 2.2-3.5 2.2 1.6 2.2 3.5Z" fill="#13241C" />
      <path d="M44.4 43.5c0 1.9-1 3.5-2.2 3.5S40 45.4 40 43.5s1-3.5 2.2-3.5 2.2 1.6 2.2 3.5Z" fill="#13241C" />
      <path d="M31 12c1-2.8 3.2-4.9 6.2-5.9" stroke="#57D3A5" strokeLinecap="round" strokeWidth="3" />
      <path d="M35.5 15c.4-1.6 1.7-3 3.4-3.6" stroke="#57D3A5" strokeLinecap="round" strokeWidth="2.5" />
    </svg>
  )
}
