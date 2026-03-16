import { HashRouter, Route, Routes } from 'react-router-dom'
import { AppShell } from './components/AppShell'
import { useAppModel } from './hooks/useAppModel'
import { DownloadsPage } from './pages/DownloadsPage'
import { HomePage } from './pages/HomePage'
import { SettingsPage } from './pages/SettingsPage'

function App() {
  const model = useAppModel()

  return (
    <HashRouter>
      <AppShell model={model}>
        <Routes>
          <Route path="/" element={<HomePage model={model} />} />
          <Route
            path="/downloads"
            element={
              <DownloadsPage
                jobs={model.jobs}
                busyJobId={model.busyJobId}
                onCancel={model.cancelJob}
                onOpenFolder={model.openOutputFolder}
                onPlayOutput={model.playOutput}
                onRemove={model.removeJob}
              />
            }
          />
          <Route
            path="/settings"
            element={
              <SettingsPage
                cacheClearing={model.cacheClearing}
                onBrowseDirectory={model.pickDownloadDirectory}
                onClearCache={model.clearCache}
                onSave={model.saveSettings}
                saveBusy={model.saveBusy}
                settings={model.settings}
              />
            }
          />
        </Routes>
      </AppShell>
    </HashRouter>
  )
}

export default App
