use std::{
  collections::BTreeSet,
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
  process::{Command, Output},
};

use chrono::Utc;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use url::Url;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::models::{AppSettings, ExportFormat, QualityOption};
use crate::state::AppPaths;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn background_command(program: impl AsRef<OsStr>) -> Command {
  let mut command = Command::new(program);
  #[cfg(windows)]
  {
    command.creation_flags(CREATE_NO_WINDOW);
  }
  command
}

pub fn now_iso() -> String {
  Utc::now().to_rfc3339()
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
  if !path.exists() {
    return Ok(None);
  }

  let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
  serde_json::from_str(&contents)
    .map(Some)
    .map_err(|error| format!("Failed to parse {}: {error}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
  let contents = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
  fs::write(path, contents).map_err(|error| error.to_string())
}

pub fn path_to_string(path: &Path) -> String {
  path.to_string_lossy().into_owned()
}

pub fn validate_youtube_url(url: &str) -> Result<Url, String> {
  let parsed = Url::parse(url).map_err(|_| "Enter a valid YouTube video URL.".to_string())?;
  let host = parsed.host_str().unwrap_or_default();

  let is_supported_host = matches!(
    host,
    "youtube.com" | "www.youtube.com" | "m.youtube.com" | "youtu.be"
  );

  if !is_supported_host {
    return Err("Only direct YouTube video URLs are supported in v1.".to_string());
  }

  Ok(parsed)
}

pub fn sanitize_filename_stem(value: &str) -> String {
  let cleaned = value
    .chars()
    .filter(|character| !matches!(character, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join("_");

  if cleaned.is_empty() {
    "watermelon_clip".to_string()
  } else {
    cleaned.chars().take(80).collect()
  }
}

pub fn resolve_command_path(command: &str) -> Result<String, String> {
  let output = background_command("where.exe")
    .arg(command)
    .output()
    .map_err(|_| format!("Missing dependency: {command}"))?;

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(path) = stdout.lines().find(|line| !line.trim().is_empty()) {
      return Ok(path.trim().to_string());
    }
  }

  let local_app_data = std::env::var("LOCALAPPDATA").map_err(|_| format!("Missing dependency: {command}"))?;
  let winget_links = Path::new(&local_app_data).join("Microsoft").join("WinGet").join("Links");
  let executable_names = if command.ends_with(".exe") {
    vec![command.to_string()]
  } else {
    vec![command.to_string(), format!("{command}.exe")]
  };

  for executable_name in executable_names {
    let candidate = winget_links.join(executable_name);
    if candidate.exists() {
      return Ok(path_to_string(&candidate));
    }
  }

  Err(format!("Missing dependency: {command}"))
}

pub fn youtube_authentication_required(message: &str) -> bool {
  let normalized = message.to_ascii_lowercase();
  normalized.contains("sign in to confirm you're not a bot")
    || normalized.contains("sign in to confirm you\u{fffd}re not a bot")
    || normalized.contains("use --cookies-from-browser or --cookies for the authentication")
    || normalized.contains("cookies for the authentication")
}

pub fn missing_browser_cookie_store(message: &str) -> bool {
  let normalized = message.to_ascii_lowercase();
  normalized.contains("could not find")
    && normalized.contains("cookies database")
}

pub fn yt_dlp_browser_candidates() -> &'static [&'static str] {
  &["edge", "chrome", "firefox", "brave", "chromium"]
}

pub fn run_yt_dlp_output_with_browser_fallback(base_args: &[String]) -> Result<Output, String> {
  let yt_dlp_path = resolve_command_path("yt-dlp")?;
  let output = background_command(&yt_dlp_path)
    .args(base_args)
    .output()
    .map_err(|error| error.to_string())?;

  if output.status.success() {
    return Ok(output);
  }

  let stderr = String::from_utf8_lossy(&output.stderr);
  if !youtube_authentication_required(&stderr) {
    return Ok(output);
  }

  let mut last_output = output;
  for browser in yt_dlp_browser_candidates() {
    let mut retry_args = vec!["--cookies-from-browser".to_string(), (*browser).to_string()];
    retry_args.extend(base_args.iter().cloned());

    let retry_output = background_command(&yt_dlp_path)
      .args(&retry_args)
      .output()
      .map_err(|error| error.to_string())?;

    if retry_output.status.success() {
      return Ok(retry_output);
    }

    last_output = retry_output;
  }

  Ok(last_output)
}

pub fn youtube_auth_guidance_message() -> String {
  "YouTube requested a bot check. Watermelon retried with browser cookies but could not authenticate. Sign in to YouTube in Edge, Chrome, Firefox, or Brave, then try again.".to_string()
}

pub fn default_settings(paths: &AppPaths) -> AppSettings {
  AppSettings {
    cache_directory: path_to_string(&paths.cache_dir),
    default_format: ExportFormat::Mp4,
    default_quality_id: "1080p".to_string(),
    download_directory: path_to_string(&paths.default_download_dir()),
    max_concurrent_downloads: 2,
    notify_on_complete: true,
    open_file_after_download: false,
    remember_clip_selection: true,
    theme: crate::models::ThemeMode::Dark,
  }
}

pub fn normalize_settings(settings: AppSettings, paths: &AppPaths) -> Result<AppSettings, String> {
  let download_directory = if settings.download_directory.trim().is_empty() {
    paths.default_download_dir()
  } else {
    PathBuf::from(settings.download_directory.trim())
  };

  fs::create_dir_all(&download_directory).map_err(|error| error.to_string())?;
  fs::create_dir_all(&paths.cache_dir).map_err(|error| error.to_string())?;

  Ok(AppSettings {
    cache_directory: path_to_string(&paths.cache_dir),
    default_format: settings.default_format,
    default_quality_id: settings.default_quality_id,
    download_directory: path_to_string(&download_directory),
    max_concurrent_downloads: settings.max_concurrent_downloads.clamp(1, 5),
    notify_on_complete: settings.notify_on_complete,
    open_file_after_download: settings.open_file_after_download,
    remember_clip_selection: settings.remember_clip_selection,
    theme: settings.theme,
  })
}

pub fn parse_quality_options(metadata: &Value) -> Vec<QualityOption> {
  let mut heights = BTreeSet::new();

  if let Some(formats) = metadata.get("formats").and_then(Value::as_array) {
    for format in formats {
      let has_video = format.get("vcodec").and_then(Value::as_str).unwrap_or("none") != "none";
      if !has_video {
        continue;
      }

      if let Some(height) = format.get("height").and_then(Value::as_u64) {
        heights.insert(height as u32);
      }
    }
  }

  let mut options = heights
    .into_iter()
    .rev()
    .take(6)
    .map(|height| QualityOption {
      height: Some(height),
      id: format!("{height}p"),
      label: quality_label(height),
    })
    .collect::<Vec<_>>();

  if options.is_empty() {
    options = vec![2160, 1080, 720, 480]
      .into_iter()
      .map(|height| QualityOption {
        height: Some(height),
        id: format!("{height}p"),
        label: quality_label(height),
      })
      .collect();
  }

  options
}

pub fn quality_height_from_id(quality_id: &str) -> Option<u32> {
  let digits = quality_id
    .chars()
    .take_while(|character| character.is_ascii_digit())
    .collect::<String>();
  digits.parse::<u32>().ok()
}

pub fn format_seconds(milliseconds: u64) -> String {
  format!("{:.3}", milliseconds as f64 / 1000.0)
}

pub fn unique_output_path(directory: &Path, stem: &str, extension: &str) -> PathBuf {
  let safe_stem = sanitize_filename_stem(stem);
  let mut candidate = directory.join(format!("{safe_stem}.{extension}"));
  let mut index = 2;

  while candidate.exists() {
    candidate = directory.join(format!("{safe_stem}_{index}.{extension}"));
    index += 1;
  }

  candidate
}

pub fn parse_yt_dlp_progress(line: &str) -> Option<(f64, Option<String>, Option<String>)> {
  if !line.contains("[download]") || !line.contains('%') {
    return None;
  }

  let percent = line[..line.find('%')?]
    .split_whitespace()
    .last()?
    .trim()
    .parse::<f64>()
    .ok()?;

  let speed = line
    .split(" at ")
    .nth(1)
    .and_then(|segment| segment.split(" ETA ").next())
    .map(|segment| segment.trim().to_string())
    .filter(|segment| !segment.is_empty());

  let eta = line
    .split(" ETA ")
    .nth(1)
    .map(|segment| segment.trim().to_string())
    .filter(|segment| !segment.is_empty());

  Some((percent, speed, eta))
}

pub fn parse_ffmpeg_progress(line: &str) -> Option<u64> {
  line
    .strip_prefix("out_time_ms=")
    .and_then(|value| value.trim().parse::<u64>().ok())
    .map(|microseconds| microseconds / 1000)
}

pub fn find_source_file(job_dir: &Path) -> Result<PathBuf, String> {
  let entries = fs::read_dir(job_dir).map_err(|error| error.to_string())?;
  for entry in entries {
    let entry = entry.map_err(|error| error.to_string())?;
    let file_name = entry.file_name().to_string_lossy().into_owned();
    if file_name.starts_with("source.") && !file_name.ends_with(".part") {
      return Ok(entry.path());
    }
  }

  Err("yt-dlp finished without producing a source file.".to_string())
}

pub fn open_in_explorer(path: &Path) -> Result<(), String> {
  background_command("explorer.exe")
    .arg(format!("/select,{}", path_to_string(path)))
    .spawn()
    .map_err(|error| error.to_string())?;
  Ok(())
}

pub fn open_folder(path: &Path) -> Result<(), String> {
  background_command("explorer.exe")
    .arg(path_to_string(path))
    .spawn()
    .map_err(|error| error.to_string())?;
  Ok(())
}

pub fn open_with_default_app(path: &Path) -> Result<(), String> {
  background_command("cmd")
    .args(["/C", "start", "", path_to_string(path).as_str()])
    .spawn()
    .map_err(|error| error.to_string())?;
  Ok(())
}

pub fn open_in_browser(url: &str) -> Result<(), String> {
  background_command("cmd")
    .args(["/C", "start", "", url])
    .spawn()
    .map_err(|error| error.to_string())?;
  Ok(())
}

pub fn cache_size(path: &Path) -> u64 {
  if !path.exists() {
    return 0;
  }

  if path.is_file() {
    return path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
  }

  let mut total = 0;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries.flatten() {
      total += cache_size(&entry.path());
    }
  }
  total
}

fn quality_label(height: u32) -> String {
  match height {
    2160 => "4K Ultra HD".to_string(),
    1440 => "1440p QHD".to_string(),
    1080 => "1080p Full HD".to_string(),
    720 => "720p HD".to_string(),
    480 => "480p SD".to_string(),
    _ => format!("{height}p"),
  }
}
