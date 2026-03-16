type TopBarProps = {
  analyzeBusy: boolean
  dependencyMessage: string | null
  onAnalyze: () => Promise<void>
  onPaste: () => Promise<void>
  onUrlChange: (value: string) => void
  url: string
  urlError: string | null
}

export function TopBar({
  analyzeBusy,
  dependencyMessage,
  onAnalyze,
  onPaste,
  onUrlChange,
  url,
  urlError,
}: TopBarProps) {
  return (
    <header className="topbar">
      <div className="topbar-input-shell">
        <input
          aria-label="YouTube URL"
          className={urlError ? 'topbar-input has-error' : 'topbar-input'}
          onChange={(event) => onUrlChange(event.target.value)}
          placeholder="Paste a YouTube URL to analyze"
          spellCheck={false}
          type="url"
          value={url}
        />
        <div className="topbar-actions">
          <button
            className="secondary-button"
            disabled={analyzeBusy}
            onClick={() => void onAnalyze()}
            type="button"
          >
            {analyzeBusy ? 'Analyzing...' : 'Analyze'}
          </button>
          <button
            className="ghost-button"
            disabled={analyzeBusy}
            onClick={() => void onPaste()}
            type="button"
          >
            Paste from Clipboard
          </button>
        </div>
      </div>

      {urlError ? <p className="inline-message error">{urlError}</p> : null}
      {dependencyMessage ? (
        <p className="inline-message warn">{dependencyMessage}</p>
      ) : null}
    </header>
  )
}
