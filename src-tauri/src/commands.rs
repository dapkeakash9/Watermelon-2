use std::{fs, path::PathBuf};

use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use crate::{
  helpers::{
    cache_size, now_iso, open_folder, open_in_explorer, open_with_default_app,
    missing_browser_cookie_store, parse_quality_options, resolve_command_path,
    run_yt_dlp_output_with_browser_fallback, sanitize_filename_stem, validate_youtube_url,
    youtube_auth_guidance_message, youtube_authentication_required,
  },
  models::{AppSettings, ClipRequest, DownloadJob, ExportFormat, JobStatus, PreviewSource, VideoAnalysis},
  state::{schedule_jobs, SharedState},
};

pub fn analyze_youtube_url_impl(url: String) -> Result<VideoAnalysis, String> {
  let parsed = validate_youtube_url(&url)?;
  let output = run_yt_dlp_output_with_browser_fallback(&[
    "--dump-single-json".to_string(),
    "--no-playlist".to_string(),
    "--no-warnings".to_string(),
    parsed.as_str().to_string(),
  ])?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(if youtube_authentication_required(&stderr) {
      youtube_auth_guidance_message()
    } else if missing_browser_cookie_store(&stderr) {
      youtube_auth_guidance_message()
    } else if stderr.trim().is_empty() {
      "yt-dlp failed to analyze the video.".to_string()
    } else {
      stderr.trim().to_string()
    });
  }

  let metadata: Value =
    serde_json::from_slice(&output.stdout).map_err(|error| format!("Invalid yt-dlp JSON: {error}"))?;

  if metadata.get("is_live").and_then(Value::as_bool).unwrap_or(false) {
    return Err("Live videos are not supported in v1.".to_string());
  }

  let title = metadata
    .get("title")
    .and_then(Value::as_str)
    .ok_or_else(|| "The selected video is missing a title.".to_string())?
    .to_string();
  let duration_seconds = metadata
    .get("duration")
    .and_then(Value::as_f64)
    .ok_or_else(|| "Unable to determine the video duration.".to_string())?;

  let canonical_url = metadata
    .get("webpage_url")
    .and_then(Value::as_str)
    .unwrap_or(parsed.as_str())
    .to_string();
  let thumbnail = metadata
    .get("thumbnail")
    .and_then(Value::as_str)
    .unwrap_or_default()
    .to_string();
  let source_id = metadata
    .get("id")
    .and_then(Value::as_str)
    .unwrap_or_default()
    .to_string();

  Ok(VideoAnalysis {
    available_formats: vec![ExportFormat::Mp4, ExportFormat::Mp3, ExportFormat::Gif],
    canonical_url,
    duration_ms: (duration_seconds * 1000.0).round() as u64,
    preview_available: true,
    quality_options: parse_quality_options(&metadata),
    source_id,
    suggested_filename: sanitize_filename_stem(&title),
    thumbnail,
    title,
  })
}

pub fn get_preview_source_impl(source: String) -> Result<PreviewSource, String> {
  let output = run_yt_dlp_output_with_browser_fallback(&[
    "-g".to_string(),
    "-f".to_string(),
    "best[ext=mp4][protocol!=m3u8]/best[ext=mp4]/best".to_string(),
    "--no-playlist".to_string(),
    source.clone(),
  ])?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(if youtube_authentication_required(&stderr) {
      youtube_auth_guidance_message()
    } else if missing_browser_cookie_store(&stderr) {
      youtube_auth_guidance_message()
    } else if stderr.trim().is_empty() {
      "Unable to resolve a preview stream.".to_string()
    } else {
      stderr.trim().to_string()
    });
  }

  let url = String::from_utf8_lossy(&output.stdout)
    .lines()
    .find(|line| !line.trim().is_empty())
    .ok_or_else(|| "yt-dlp returned an empty preview source.".to_string())?
    .trim()
    .to_string();

  Ok(PreviewSource {
    kind: "remote".to_string(),
    mime_type: "video/mp4".to_string(),
    url,
  })
}

pub fn create_clip_job_impl(request: ClipRequest, state: &SharedState) -> Result<DownloadJob, String> {
  validate_youtube_url(&request.source_url)?;
  resolve_command_path("yt-dlp")?;
  resolve_command_path("ffmpeg")?;

  if request.selection.end_ms <= request.selection.start_ms {
    return Err("The clip end point must be after the start point.".to_string());
  }

  let job = DownloadJob {
    bytes_downloaded: None,
    bytes_total: None,
    created_at: now_iso(),
    error: None,
    eta_text: None,
    format: request.format.clone(),
    id: Uuid::new_v4().to_string(),
    output_file_name: Some(sanitize_filename_stem(&request.filename)),
    percent: 0.0,
    quality_label: Some(request.quality_id.clone()),
    selection: request.selection.clone(),
    source_url: request.source_url.clone(),
    speed_text: None,
    status: JobStatus::Queued,
    target_path: None,
    thumbnail: request.thumbnail.clone(),
    title: request.title.clone(),
    updated_at: now_iso(),
  };

  state.0.push_job(job.clone())?;
  schedule_jobs(state.0.clone());
  Ok(job)
}

pub fn list_jobs_impl(state: &SharedState) -> Result<Vec<DownloadJob>, String> {
  state.0.list_jobs()
}

pub fn get_job_impl(job_id: String, state: &SharedState) -> Result<DownloadJob, String> {
  state.0.get_job(&job_id)
}

pub fn cancel_job_impl(job_id: String, state: &SharedState) -> Result<(), String> {
  if state.0.cancel_running_job(&job_id)? {
    return Ok(());
  }

  state.0.update_job(&job_id, |job| {
    if job.status == JobStatus::Queued {
      job.status = JobStatus::Canceled;
      job.error = Some("Canceled by user.".to_string());
    }
  })?;

  Ok(())
}

pub fn remove_job_impl(job_id: String, state: &SharedState) -> Result<(), String> {
  if state.0.is_running(&job_id)? {
    return Err("Cancel the running job before removing it.".to_string());
  }

  let job = state.0.get_job(&job_id)?;
  if !matches!(job.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Canceled) {
    return Err("Only completed, failed, or canceled jobs can be removed.".to_string());
  }

  state.0.remove_job(&job_id)
}

pub fn open_output_impl(job_id: String, state: &SharedState) -> Result<(), String> {
  let job = state.0.get_job(&job_id)?;
  let target_path = job
    .target_path
    .ok_or_else(|| "The selected job has no output path yet.".to_string())?;
  let path = PathBuf::from(target_path);

  if !path.exists() {
    return Err("The exported file no longer exists on disk.".to_string());
  }

  open_in_explorer(&path)
}

pub fn play_output_impl(job_id: String, state: &SharedState) -> Result<(), String> {
  let job = state.0.get_job(&job_id)?;
  let target_path = job
    .target_path
    .ok_or_else(|| "The selected job has no output path yet.".to_string())?;
  let path = PathBuf::from(target_path);

  if !path.exists() {
    return Err("The exported file no longer exists on disk.".to_string());
  }

  open_with_default_app(&path)
}

pub fn open_output_folder_impl(job_id: String, state: &SharedState) -> Result<(), String> {
  let job = state.0.get_job(&job_id)?;
  let settings = state.0.get_settings()?;

  let folder_path = job
    .target_path
    .as_deref()
    .map(PathBuf::from)
    .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
    .unwrap_or_else(|| PathBuf::from(settings.download_directory));

  if !folder_path.exists() {
    fs::create_dir_all(&folder_path).map_err(|error| error.to_string())?;
  }

  open_folder(&folder_path)
}

pub fn get_settings_impl(state: &SharedState) -> Result<AppSettings, String> {
  state.0.get_settings()
}

pub fn save_settings_impl(settings: AppSettings, state: &SharedState) -> Result<AppSettings, String> {
  let stored = state.0.set_settings(settings)?;
  schedule_jobs(state.0.clone());
  Ok(stored)
}

pub fn pick_download_directory_impl() -> Result<Option<String>, String> {
  Ok(
    rfd::FileDialog::new()
      .pick_folder()
      .map(|path| path.to_string_lossy().into_owned()),
  )
}

pub fn clear_cache_impl(state: &SharedState) -> Result<u64, String> {
  if state.0.has_running_jobs()? {
    return Err("Wait for active downloads to finish before clearing the cache.".to_string());
  }

  let removed_bytes = cache_size(&state.0.paths.cache_dir);
  if state.0.paths.cache_dir.exists() {
    fs::remove_dir_all(&state.0.paths.cache_dir).map_err(|error| error.to_string())?;
  }
  fs::create_dir_all(&state.0.paths.cache_dir).map_err(|error| error.to_string())?;
  state.0.persist_settings()?;
  Ok(removed_bytes)
}

#[tauri::command]
pub fn analyze_youtube_url(url: String) -> Result<VideoAnalysis, String> {
  analyze_youtube_url_impl(url)
}

#[tauri::command]
pub fn get_preview_source(source: String) -> Result<PreviewSource, String> {
  get_preview_source_impl(source)
}

#[tauri::command]
pub fn create_clip_job(request: ClipRequest, state: State<'_, SharedState>) -> Result<DownloadJob, String> {
  create_clip_job_impl(request, state.inner())
}

#[tauri::command]
pub fn list_jobs(state: State<'_, SharedState>) -> Result<Vec<DownloadJob>, String> {
  list_jobs_impl(state.inner())
}

#[tauri::command]
pub fn get_job(job_id: String, state: State<'_, SharedState>) -> Result<DownloadJob, String> {
  get_job_impl(job_id, state.inner())
}

#[tauri::command]
pub fn cancel_job(job_id: String, state: State<'_, SharedState>) -> Result<(), String> {
  cancel_job_impl(job_id, state.inner())
}

#[tauri::command]
pub fn remove_job(job_id: String, state: State<'_, SharedState>) -> Result<(), String> {
  remove_job_impl(job_id, state.inner())
}

#[tauri::command]
pub fn open_output(job_id: String, state: State<'_, SharedState>) -> Result<(), String> {
  open_output_impl(job_id, state.inner())
}

#[tauri::command]
pub fn play_output(job_id: String, state: State<'_, SharedState>) -> Result<(), String> {
  play_output_impl(job_id, state.inner())
}

#[tauri::command]
pub fn open_output_folder(job_id: String, state: State<'_, SharedState>) -> Result<(), String> {
  open_output_folder_impl(job_id, state.inner())
}

#[tauri::command]
pub fn get_settings(state: State<'_, SharedState>) -> Result<AppSettings, String> {
  get_settings_impl(state.inner())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, state: State<'_, SharedState>) -> Result<AppSettings, String> {
  save_settings_impl(settings, state.inner())
}

#[tauri::command]
pub fn pick_download_directory() -> Result<Option<String>, String> {
  pick_download_directory_impl()
}

#[tauri::command]
pub fn clear_cache(state: State<'_, SharedState>) -> Result<u64, String> {
  clear_cache_impl(state.inner())
}
