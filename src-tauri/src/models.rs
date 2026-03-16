use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
  Mp4,
  Mp3,
  Gif,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
  Queued,
  Analyzing,
  Downloading,
  Processing,
  Completed,
  Failed,
  Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
  Light,
  Dark,
  System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityOption {
  pub height: Option<u32>,
  pub id: String,
  pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAnalysis {
  pub available_formats: Vec<ExportFormat>,
  pub canonical_url: String,
  pub duration_ms: u64,
  pub preview_available: bool,
  pub quality_options: Vec<QualityOption>,
  pub source_id: String,
  pub suggested_filename: String,
  pub thumbnail: String,
  pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSource {
  pub kind: String,
  pub mime_type: String,
  pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipSelection {
  pub end_ms: u64,
  pub start_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipRequest {
  pub filename: String,
  pub format: ExportFormat,
  pub quality_id: String,
  pub selection: ClipSelection,
  pub source_id: String,
  pub source_url: String,
  pub thumbnail: String,
  pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJob {
  pub bytes_downloaded: Option<u64>,
  pub bytes_total: Option<u64>,
  pub created_at: String,
  pub error: Option<String>,
  pub eta_text: Option<String>,
  pub format: ExportFormat,
  pub id: String,
  pub output_file_name: Option<String>,
  pub percent: f64,
  pub quality_label: Option<String>,
  pub selection: ClipSelection,
  pub source_url: String,
  pub speed_text: Option<String>,
  pub status: JobStatus,
  pub target_path: Option<String>,
  pub thumbnail: String,
  pub title: String,
  pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
  pub cache_directory: String,
  pub default_format: ExportFormat,
  pub default_quality_id: String,
  pub download_directory: String,
  pub max_concurrent_downloads: usize,
  pub notify_on_complete: bool,
  pub open_file_after_download: bool,
  pub remember_clip_selection: bool,
  pub theme: ThemeMode,
}
