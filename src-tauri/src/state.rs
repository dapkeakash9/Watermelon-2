use std::{
  collections::HashMap,
  fs,
  io::{BufRead, BufReader, Read},
  path::{Path, PathBuf},
  process::Stdio,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use crate::{
  helpers::{
    background_command, default_settings, find_source_file, format_seconds, normalize_settings, now_iso,
    missing_browser_cookie_store,
    open_in_explorer, parse_ffmpeg_progress, parse_yt_dlp_progress, path_to_string,
    quality_height_from_id, read_json, resolve_command_path, unique_output_path, write_json,
    yt_dlp_browser_candidates, youtube_auth_guidance_message, youtube_authentication_required,
  },
  models::{AppSettings, DownloadJob, ExportFormat, JobStatus},
};

#[derive(Debug, Clone)]
pub struct AppPaths {
  pub cache_dir: PathBuf,
  pub data_dir: PathBuf,
  pub jobs_file: PathBuf,
  pub settings_file: PathBuf,
}

pub struct JobControl {
  cancel: AtomicBool,
  pid: Mutex<Option<u32>>,
}

pub struct AppState {
  pub paths: AppPaths,
  jobs: Mutex<Vec<DownloadJob>>,
  running: Mutex<HashMap<String, Arc<JobControl>>>,
  settings: Mutex<AppSettings>,
}

#[derive(Clone)]
pub struct SharedState(pub Arc<AppState>);

impl JobControl {
  pub fn new() -> Self {
    Self {
      cancel: AtomicBool::new(false),
      pid: Mutex::new(None),
    }
  }

  pub fn set_pid(&self, pid: Option<u32>) {
    if let Ok(mut stored_pid) = self.pid.lock() {
      *stored_pid = pid;
    }
  }

  pub fn is_canceled(&self) -> bool {
    self.cancel.load(Ordering::SeqCst)
  }

  pub fn cancel(&self) {
    self.cancel.store(true, Ordering::SeqCst);
    if let Ok(stored_pid) = self.pid.lock() {
      if let Some(pid) = *stored_pid {
        let _ = background_command("taskkill")
          .args(["/PID", &pid.to_string(), "/T", "/F"])
          .stdout(Stdio::null())
          .stderr(Stdio::null())
          .status();
      }
    }
  }
}

impl AppPaths {
  pub fn new() -> Result<Self, String> {
    let data_root = dirs::data_local_dir()
      .or_else(dirs::data_dir)
      .ok_or_else(|| "Unable to resolve the application data directory.".to_string())?
      .join("Watermelon 2");
    let cache_root = dirs::cache_dir()
      .ok_or_else(|| "Unable to resolve the cache directory.".to_string())?
      .join("Watermelon 2");

    fs::create_dir_all(&data_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&cache_root).map_err(|error| error.to_string())?;

    Ok(Self {
      data_dir: data_root.clone(),
      cache_dir: cache_root,
      jobs_file: data_root.join("jobs.json"),
      settings_file: data_root.join("settings.json"),
    })
  }

  pub fn default_download_dir(&self) -> PathBuf {
    let root = dirs::download_dir().unwrap_or_else(|| self.data_dir.join("Downloads"));
    root.join("Watermelon 2")
  }
}

impl AppState {
  pub fn new() -> Result<Self, String> {
    let paths = AppPaths::new()?;
    let settings =
      normalize_settings(read_json(&paths.settings_file)?.unwrap_or_else(|| default_settings(&paths)), &paths)?;

    let jobs = read_json::<Vec<DownloadJob>>(&paths.jobs_file)?
      .unwrap_or_default()
      .into_iter()
      .map(reset_interrupted_job)
      .collect::<Vec<_>>();

    let state = Self {
      paths,
      jobs: Mutex::new(jobs),
      running: Mutex::new(HashMap::new()),
      settings: Mutex::new(settings),
    };

    state.persist_jobs()?;
    state.persist_settings()?;
    Ok(state)
  }

  pub fn list_jobs(&self) -> Result<Vec<DownloadJob>, String> {
    let mut jobs = self.jobs.lock().map_err(|_| "Jobs lock poisoned.".to_string())?.clone();
    jobs.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(jobs)
  }

  pub fn get_job(&self, job_id: &str) -> Result<DownloadJob, String> {
    self
      .jobs
      .lock()
      .map_err(|_| "Jobs lock poisoned.".to_string())?
      .iter()
      .find(|job| job.id == job_id)
      .cloned()
      .ok_or_else(|| "Job not found.".to_string())
  }

  pub fn push_job(&self, job: DownloadJob) -> Result<(), String> {
    let mut jobs = self.jobs.lock().map_err(|_| "Jobs lock poisoned.".to_string())?;
    jobs.push(job);
    let snapshot = jobs.clone();
    drop(jobs);
    write_json(&self.paths.jobs_file, &snapshot)
  }

  pub fn update_job<F>(&self, job_id: &str, updater: F) -> Result<DownloadJob, String>
  where
    F: FnOnce(&mut DownloadJob),
  {
    let mut jobs = self.jobs.lock().map_err(|_| "Jobs lock poisoned.".to_string())?;
    let updated = {
      let job = jobs
        .iter_mut()
        .find(|job| job.id == job_id)
        .ok_or_else(|| "Job not found.".to_string())?;
      updater(job);
      job.updated_at = now_iso();
      job.clone()
    };
    let snapshot = jobs.clone();
    drop(jobs);
    write_json(&self.paths.jobs_file, &snapshot)?;
    Ok(updated)
  }

  pub fn remove_job(&self, job_id: &str) -> Result<(), String> {
    let mut jobs = self.jobs.lock().map_err(|_| "Jobs lock poisoned.".to_string())?;
    let previous_len = jobs.len();
    jobs.retain(|job| job.id != job_id);
    if jobs.len() == previous_len {
      return Err("Job not found.".to_string());
    }
    let snapshot = jobs.clone();
    drop(jobs);
    write_json(&self.paths.jobs_file, &snapshot)
  }

  pub fn persist_jobs(&self) -> Result<(), String> {
    let snapshot = self.jobs.lock().map_err(|_| "Jobs lock poisoned.".to_string())?.clone();
    write_json(&self.paths.jobs_file, &snapshot)
  }

  pub fn get_settings(&self) -> Result<AppSettings, String> {
    Ok(self.settings.lock().map_err(|_| "Settings lock poisoned.".to_string())?.clone())
  }

  pub fn set_settings(&self, settings: AppSettings) -> Result<AppSettings, String> {
    let normalized = normalize_settings(settings, &self.paths)?;
    {
      let mut stored = self.settings.lock().map_err(|_| "Settings lock poisoned.".to_string())?;
      *stored = normalized.clone();
    }
    self.persist_settings()?;
    Ok(normalized)
  }

  pub fn persist_settings(&self) -> Result<(), String> {
    let snapshot = self
      .settings
      .lock()
      .map_err(|_| "Settings lock poisoned.".to_string())?
      .clone();
    write_json(&self.paths.settings_file, &snapshot)
  }

  pub fn is_running(&self, job_id: &str) -> Result<bool, String> {
    Ok(
      self
        .running
        .lock()
        .map_err(|_| "Running jobs lock poisoned.".to_string())?
        .contains_key(job_id),
    )
  }

  pub fn cancel_running_job(&self, job_id: &str) -> Result<bool, String> {
    let control = self
      .running
      .lock()
      .map_err(|_| "Running jobs lock poisoned.".to_string())?
      .get(job_id)
      .cloned();

    if let Some(control) = control {
      control.cancel();
      return Ok(true);
    }

    Ok(false)
  }

  pub fn has_running_jobs(&self) -> Result<bool, String> {
    Ok(
      !self
        .running
        .lock()
        .map_err(|_| "Running jobs lock poisoned.".to_string())?
        .is_empty(),
    )
  }
}

impl SharedState {
  pub fn new() -> Result<Self, String> {
    Ok(Self(Arc::new(AppState::new()?)))
  }
}

pub fn schedule_jobs(state: Arc<AppState>) {
  loop {
    let next_job_id = {
      let settings = match state.settings.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return,
      };
      let jobs = match state.jobs.lock() {
        Ok(guard) => guard,
        Err(_) => return,
      };
      let running = match state.running.lock() {
        Ok(guard) => guard,
        Err(_) => return,
      };

      if running.len() >= settings.max_concurrent_downloads {
        None
      } else {
        jobs
          .iter()
          .find(|job| job.status == JobStatus::Queued && !running.contains_key(&job.id))
          .map(|job| job.id.clone())
      }
    };

    let Some(job_id) = next_job_id else {
      break;
    };

    let control = Arc::new(JobControl::new());
    if let Ok(mut running) = state.running.lock() {
      running.insert(job_id.clone(), control.clone());
    }

    let cloned_state = state.clone();
    tauri::async_runtime::spawn_blocking(move || {
      process_job(cloned_state.clone(), job_id.clone(), control.clone());
      if let Ok(mut running) = cloned_state.running.lock() {
        running.remove(&job_id);
      }
      schedule_jobs(cloned_state);
    });
  }
}

fn process_job(state: Arc<AppState>, job_id: String, control: Arc<JobControl>) {
  let execution = || -> Result<(), String> {
    let job = state.get_job(&job_id)?;
    let job_dir = state.paths.cache_dir.join("jobs").join(&job.id);
    fs::create_dir_all(&job_dir).map_err(|error| error.to_string())?;

    let source_path = download_source(&state, &job, &job_dir, &control)?;
    export_clip(&state, &job, &source_path, &control)?;

    if job_dir.exists() {
      let _ = fs::remove_dir_all(&job_dir);
    }
    Ok(())
  };

  match execution() {
    Ok(()) => {}
    Err(error) if control.is_canceled() || error == "canceled" => {
      let _ = state.update_job(&job_id, |job| {
        job.status = JobStatus::Canceled;
        job.error = Some("Canceled by user.".to_string());
        job.speed_text = None;
        job.eta_text = None;
      });
    }
    Err(error) => {
      let _ = state.update_job(&job_id, |job| {
        job.status = JobStatus::Failed;
        job.error = Some(error);
        job.speed_text = None;
        job.eta_text = None;
      });
    }
  }
}

fn reset_interrupted_job(mut job: DownloadJob) -> DownloadJob {
  if matches!(
    job.status,
    JobStatus::Analyzing | JobStatus::Downloading | JobStatus::Processing
  ) {
    job.status = JobStatus::Queued;
    job.percent = 0.0;
    job.speed_text = None;
    job.eta_text = None;
    job.error = Some("Recovered after app restart.".to_string());
    job.updated_at = now_iso();
  }
  job
}

fn download_source(
  state: &AppState,
  job: &DownloadJob,
  job_dir: &Path,
  control: &JobControl,
) -> Result<PathBuf, String> {
  state.update_job(&job.id, |stored_job| {
    stored_job.status = JobStatus::Downloading;
    stored_job.percent = 0.0;
    stored_job.error = None;
  })?;

  let yt_dlp_path = resolve_command_path("yt-dlp")?;
  let output_template = job_dir.join("source.%(ext)s");
  let base_args = vec![
    "--newline".to_string(),
    "--no-playlist".to_string(),
    "--merge-output-format".to_string(),
    "mp4".to_string(),
    "-f".to_string(),
    "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string(),
    "-o".to_string(),
    output_template.to_string_lossy().into_owned(),
    job.source_url.clone(),
  ];

  let mut attempt_browsers = vec![None];
  attempt_browsers.extend(yt_dlp_browser_candidates().iter().map(|browser| Some(*browser)));

  let mut saw_auth_error = false;

  for cookie_browser in attempt_browsers {
    let mut args = Vec::new();
    if let Some(browser) = cookie_browser {
      args.push("--cookies-from-browser".to_string());
      args.push(browser.to_string());
    }
    args.extend(base_args.iter().cloned());

    let mut child = background_command(&yt_dlp_path)
      .args(&args)
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|error| error.to_string())?;

    control.set_pid(Some(child.id()));

    let mut last_message = String::new();
    if let Some(stderr) = child.stderr.take() {
      for line in BufReader::new(stderr).lines() {
        let line = line.map_err(|error| error.to_string())?;
        if let Some((percent, speed, eta)) = parse_yt_dlp_progress(&line) {
          let _ = state.update_job(&job.id, |stored_job| {
            stored_job.status = JobStatus::Downloading;
            stored_job.percent = percent.clamp(0.0, 99.0);
            stored_job.speed_text = speed.clone();
            stored_job.eta_text = eta.clone();
          });
        } else if !line.trim().is_empty() {
          last_message = line.clone();
        }

        if control.is_canceled() {
          break;
        }
      }
    }

    let status = child.wait().map_err(|error| error.to_string())?;
    control.set_pid(None);

    if control.is_canceled() {
      return Err("canceled".to_string());
    }

    if status.success() {
      return find_source_file(job_dir);
    }

    if youtube_authentication_required(&last_message) || missing_browser_cookie_store(&last_message) {
      saw_auth_error = true;
      continue;
    }

    return Err(if last_message.is_empty() {
      "yt-dlp failed while downloading the source media.".to_string()
    } else {
      last_message
    });
  }

  if saw_auth_error {
    return Err(youtube_auth_guidance_message());
  }

  Err("yt-dlp failed while downloading the source media.".to_string())
}

fn export_clip(
  state: &AppState,
  job: &DownloadJob,
  source_path: &Path,
  control: &JobControl,
) -> Result<(), String> {
  state.update_job(&job.id, |stored_job| {
    stored_job.status = JobStatus::Processing;
    stored_job.percent = 0.0;
    stored_job.speed_text = None;
    stored_job.eta_text = None;
    stored_job.error = None;
  })?;

  let settings = state.get_settings()?;
  let ffmpeg_path = resolve_command_path("ffmpeg")?;
  let output_directory = PathBuf::from(settings.download_directory.clone());
  fs::create_dir_all(&output_directory).map_err(|error| error.to_string())?;

  let (extension, mut args) = match job.format {
    ExportFormat::Mp4 => (
      "mp4",
      {
        let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-y".to_string(),
        "-ss".to_string(),
        format_seconds(job.selection.start_ms),
        "-t".to_string(),
        format_seconds(job.selection.end_ms - job.selection.start_ms),
        "-i".to_string(),
        path_to_string(source_path),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "faster".to_string(),
        "-crf".to_string(),
        "23".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
        ];

        if let Some(height) = job.quality_label.as_deref().and_then(quality_height_from_id) {
          args.push("-vf".to_string());
          args.push(format!("scale=-2:{height}:flags=lanczos"));
        }

        args
      },
    ),
    ExportFormat::Mp3 => (
      "mp3",
      vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-y".to_string(),
        "-ss".to_string(),
        format_seconds(job.selection.start_ms),
        "-t".to_string(),
        format_seconds(job.selection.end_ms - job.selection.start_ms),
        "-i".to_string(),
        path_to_string(source_path),
        "-vn".to_string(),
        "-c:a".to_string(),
        "libmp3lame".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
      ],
    ),
    ExportFormat::Gif => (
      "gif",
      vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-y".to_string(),
        "-ss".to_string(),
        format_seconds(job.selection.start_ms),
        "-t".to_string(),
        format_seconds(job.selection.end_ms - job.selection.start_ms),
        "-i".to_string(),
        path_to_string(source_path),
        "-vf".to_string(),
        "fps=12,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse".to_string(),
      ],
    ),
  };

  let output_path = unique_output_path(
    &output_directory,
    job.output_file_name.as_deref().unwrap_or("watermelon_clip"),
    extension,
  );

  args.extend([
    "-progress".to_string(),
    "pipe:1".to_string(),
    "-nostats".to_string(),
    path_to_string(&output_path),
  ]);

  let mut child = background_command(ffmpeg_path)
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| error.to_string())?;

  control.set_pid(Some(child.id()));

  if let Some(stdout) = child.stdout.take() {
    let total_duration_ms = (job.selection.end_ms - job.selection.start_ms).max(1);
    for line in BufReader::new(stdout).lines() {
      let line = line.map_err(|error| error.to_string())?;
      if let Some(out_time_ms) = parse_ffmpeg_progress(&line) {
        let percent = ((out_time_ms as f64 / total_duration_ms as f64) * 100.0).clamp(0.0, 99.0);
        let _ = state.update_job(&job.id, |stored_job| {
          stored_job.status = JobStatus::Processing;
          stored_job.percent = percent;
        });
      }
      if control.is_canceled() {
        break;
      }
    }
  }

  let mut stderr_text = String::new();
  if let Some(mut stderr) = child.stderr.take() {
    let _ = stderr.read_to_string(&mut stderr_text);
  }

  let status = child.wait().map_err(|error| error.to_string())?;
  control.set_pid(None);

  if control.is_canceled() {
    return Err("canceled".to_string());
  }

  if !status.success() {
    let detail = stderr_text.trim();
    return Err(if detail.is_empty() {
      "ffmpeg failed while exporting the clip.".to_string()
    } else {
      detail.to_string()
    });
  }

  state.update_job(&job.id, |stored_job| {
    stored_job.status = JobStatus::Completed;
    stored_job.percent = 100.0;
    stored_job.target_path = Some(path_to_string(&output_path));
    stored_job.error = None;
    stored_job.speed_text = None;
    stored_job.eta_text = None;
  })?;

  if settings.open_file_after_download {
    let _ = open_in_explorer(&output_path);
  }

  Ok(())
}
