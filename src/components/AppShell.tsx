import { useState, type ReactNode } from 'react'
import { NavLink, useLocation } from 'react-router-dom'
import type { UseAppModel } from '../hooks/useAppModel'
import {
  DownloadIcon,
  HomeIcon,
  SettingsIcon,
} from './Icons'
import { QueueStrip } from './QueueStrip'
import { TopBar } from './TopBar'

type AppShellProps = {
  children: ReactNode
  model: UseAppModel
}

const navItems = [
  { to: '/', label: 'Home', icon: HomeIcon },
  { to: '/downloads', label: 'Downloads', icon: DownloadIcon },
  { to: '/settings', label: 'Settings', icon: SettingsIcon },
]

export function AppShell({ children, model }: AppShellProps) {
  const [aboutOpen, setAboutOpen] = useState(false)
  const location = useLocation()
  const activeDownloads = model.jobs.filter((job) =>
    ['queued', 'analyzing', 'downloading', 'processing'].includes(job.status),
  ).length

  return (
    <>
      <div className="app-shell">
        <aside className="sidebar">
          <div className="brand-mark">
            <button
              aria-label="About Watermelon 2"
              className="brand-icon brand-button"
              onClick={() => setAboutOpen(true)}
              title="About Watermelon 2"
              type="button"
            >
              <img alt="Watermelon 2" src="/watermelon.ico" />
            </button>
          </div>

          <nav className="sidebar-nav" aria-label="Primary">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                aria-label={item.label}
                className={({ isActive }) =>
                  isActive ? 'sidebar-link active' : 'sidebar-link'
                }
                title={item.label}
                to={item.to}
              >
                <span className="sidebar-link-icon">
                  <item.icon title={item.label} />
                </span>
                {item.to === '/downloads' && activeDownloads > 0 ? (
                  <span className="sidebar-badge">{activeDownloads}</span>
                ) : null}
              </NavLink>
            ))}
          </nav>

          <div className="sidebar-footer">
            <button
              aria-label="About Watermelon 2"
              className="sidebar-link about-link"
              onClick={() => setAboutOpen(true)}
              title="About"
              type="button"
            >
              <span className="sidebar-link-icon sidebar-link-glyph">@</span>
            </button>
          </div>
        </aside>

        <div className="app-main">
          {location.pathname === '/' ? (
            <TopBar
              analyzeBusy={model.analyzeBusy}
              dependencyMessage={model.dependencyMessage}
              onAnalyze={model.analyzeUrl}
              onPaste={model.pasteFromClipboard}
              onUrlChange={model.setUrl}
              url={model.url}
              urlError={model.urlError}
            />
          ) : null}
          <main className="page-content">{children}</main>
          {location.pathname === '/' ? (
            <QueueStrip
              busyJobId={model.busyJobId}
              jobs={model.jobs}
              onCancel={model.cancelJob}
              onOpenFolder={model.openOutputFolder}
              onPlayOutput={model.playOutput}
            />
          ) : null}
        </div>
      </div>

      {aboutOpen ? (
        <div
          aria-labelledby="about-watermelon-title"
          aria-modal="true"
          className="modal-backdrop"
          onClick={() => setAboutOpen(false)}
          role="dialog"
        >
          <div className="about-modal" onClick={(event) => event.stopPropagation()}>
            <div className="about-banner">
              <img alt="Watermelon 2 icon" src="/watermelon.ico" />
            </div>
            <div className="about-content">
              <h2 id="about-watermelon-title">Watermelon 2</h2>
              <p>Version 2.0</p>
              <p>Created by Akash Dapke</p>
              <p>GitHub: github.com/dapkeakash9</p>
              <p>AI Artist • Prompt Engineer</p>
              <p>Motion Graphics Designer</p>
              <p>&copy; 2026 Watermelon Project</p>
            </div>
            <div className="about-actions">
              <button className="primary-button" onClick={() => setAboutOpen(false)} type="button">
                Close
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </>
  )
}
