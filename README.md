# Watermelon 2

Windows-first YouTube clip extraction with a browser UI and a background tray host.

Watermelon 2 analyzes a YouTube video, loads a playable preview, lets you mark exact `IN` and `OUT` points, and exports the selected range as `MP4`, `MP3`, or `GIF`. The interface runs in your default browser while a lightweight Tauri host stays in the Windows notification area.

## Highlights

- Browser-based UI with a Windows tray/background host
- Analyze direct YouTube video URLs with `yt-dlp`
- Playable preview for clip selection
- Precise `IN` / `OUT` timeline editing
- Export as `MP4`, `MP3`, or `GIF`
- Persistent download queue and job history
- Download actions for play, open folder, and remove
- Basic settings for theme, storage, defaults, notifications, and queue concurrency
- Windows-oriented packaging with `.exe`, `.msi`, and NSIS installer output

## Product Behavior

Watermelon 2 is not a normal desktop windowed app.

- The app host runs in the background
- The UI opens in your default browser
- The tray icon remains in the Windows notification area
- The Tauri host does not need to stay visible in the taskbar

This design keeps the UI lightweight while still allowing local filesystem access, background jobs, tray integration, and native packaging.

## Tech Stack

- `React 19`
- `Vite`
- `TypeScript`
- `Tauri 2`
- `Rust`
- `Axum`
- `yt-dlp`
- `ffmpeg`

## Core Workflow

1. Paste a YouTube video URL on the Home screen
2. Analyze the source
3. Load the preview and inspect the timeline
4. Set `IN` and `OUT` markers
5. Choose export format and quality
6. Download the clip
7. Track progress from Downloads and the queue

## Screens

- `Home`
  - URL analysis
  - video preview
  - timeline selection
  - export controls
- `Downloads`
  - active/completed jobs
  - search and filters
  - play / open folder / remove actions
- `Settings`
  - theme
  - download directory
  - default format and quality
  - download behavior
  - concurrency and cache controls

## Local Requirements

Watermelon 2 depends on local media tools:

- `yt-dlp`
- `ffmpeg`
- Rust toolchain
- Visual Studio Build Tools for Tauri builds on Windows

The project already includes a Windows helper script at `scripts\\with-vs-dev.cmd` so npm-based Tauri commands can run inside a proper MSVC environment.

## Development

```bash
npm install
npm run dev
```

## Useful Commands

```bash
npm run test
npm run build
npm run tauri:check
npm run tauri:dev
npm run tauri:build:debug
npm run tauri:build
```

## Build Outputs

Debug builds are generated under:

- `src-tauri/target/debug/watermelon_2.exe`
- `src-tauri/target/debug/bundle/msi/`
- `src-tauri/target/debug/bundle/nsis/`

## Project Structure

```text
src/                  React UI
src/components/       shared UI components
src/pages/            Home, Downloads, Settings
src/lib/              browser API client and helpers
src/hooks/            app state model
src-tauri/src/        Rust commands, queue, tray host, HTTP server
src-tauri/icons/      packaged app icons
scripts/              Windows toolchain helper
```

## Notes on YouTube Access

Some videos trigger YouTube bot checks. Watermelon 2 retries with browser cookies from common Windows browsers. If access still fails:

- sign in to YouTube in Edge, Chrome, Firefox, or Brave
- retry analysis

This project only targets direct YouTube video URLs. Playlists, channels, live streams, and sign-in-gated edge cases are not part of the current MVP target.

## Current Scope

Implemented:

- browser-hosted UI
- tray/background app host
- analysis and preview resolution
- export queue
- persistent jobs and settings
- Windows packaging

Deferred:

- pause/resume queue controls
- advanced account/settings flows
- batch download workflows

## Credits

Created by **Akash Dapke**  
GitHub: **github.com/dapkeakash9**

AI Artist • Prompt Engineer  
Motion Graphics Designer

## License

No license file is currently configured in this repository.
