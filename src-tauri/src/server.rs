use std::io::ErrorKind;

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
#[cfg(not(dev))]
use include_dir::{include_dir, Dir};
#[cfg(not(dev))]
use axum::http::{header, HeaderValue, Uri};
use serde::{Deserialize, Serialize};
use tauri::{
  menu::{MenuBuilder, MenuItemBuilder},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  App, AppHandle, Emitter,
};
use tokio::{net::TcpListener, runtime::Builder};

use crate::{
  commands,
  helpers::open_in_browser,
  models::{AppSettings, ClipRequest},
  state::SharedState,
};

const SERVER_PORT: u16 = 3210;

#[cfg(not(dev))]
static WEB_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist");

#[derive(Debug, Serialize)]
struct ApiError {
  message: String,
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    (StatusCode::BAD_REQUEST, Json(self)).into_response()
  }
}

#[derive(Debug, Deserialize)]
struct AnalyzeRequest {
  url: String,
}

#[derive(Debug, Deserialize)]
struct PreviewRequest {
  source: String,
}

pub fn browser_entry_url() -> String {
  if cfg!(dev) {
    "http://localhost:1420".to_string()
  } else {
    format!("http://127.0.0.1:{SERVER_PORT}")
  }
}

pub fn start_background_server(state: SharedState) -> Result<(), String> {
  let address = format!("127.0.0.1:{SERVER_PORT}");
  let listener = match std::net::TcpListener::bind(&address) {
    Ok(listener) => listener,
    Err(error) if error.kind() == ErrorKind::AddrInUse => {
      return Ok(());
    }
    Err(error) => return Err(error.to_string()),
  };

  listener.set_nonblocking(true).map_err(|error| error.to_string())?;
  let router = build_router(state);

  std::thread::spawn(move || {
    let runtime = match Builder::new_current_thread().enable_all().build() {
      Ok(runtime) => runtime,
      Err(error) => {
        log::error!("failed to create background server runtime: {error}");
        return;
      }
    };

    runtime.block_on(async move {
      let listener = match TcpListener::from_std(listener) {
        Ok(listener) => listener,
        Err(error) => {
          log::error!("failed to convert background listener: {error}");
          return;
        }
      };

      if let Err(error) = axum::serve(listener, router).await {
        log::error!("background server stopped: {error}");
      }
    });
  });

  Ok(())
}

pub fn configure_background_host(app: &App, launch_url: String) -> Result<(), Box<dyn std::error::Error>> {
  let open_item = MenuItemBuilder::with_id("open", "Open Watermelon 2").build(app)?;
  let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
  let menu = MenuBuilder::new(app).items(&[&open_item, &quit_item]).build()?;

  let mut tray = TrayIconBuilder::new()
    .tooltip("Watermelon 2 is running in the background")
    .menu(&menu);

  if let Some(icon) = app.default_window_icon().cloned() {
    tray = tray.icon(icon);
  }

  let menu_launch_url = launch_url.clone();
  let tray_launch_url = launch_url.clone();

  tray
    .on_menu_event(move |app, event| match event.id().as_ref() {
      "open" => {
        let _ = open_in_browser(&menu_launch_url);
      }
      "quit" => {
        app.exit(0);
      }
      _ => {}
    })
    .on_tray_icon_event(move |_tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        let _ = open_in_browser(&tray_launch_url);
      }
    })
    .build(app)?;

  Ok(())
}

pub fn emit_background_ready(app: &AppHandle, launch_url: &str) {
  let _ = app.emit("watermelon2://background-ready", launch_url.to_string());
}

fn build_router(state: SharedState) -> Router {
  let api = Router::new()
    .route("/analyze", post(analyze))
    .route("/preview", post(preview))
    .route("/jobs", get(list_jobs).post(create_job))
    .route("/jobs/{job_id}", get(get_job).delete(remove_job))
    .route("/jobs/{job_id}/cancel", post(cancel_job))
    .route("/jobs/{job_id}/play", post(play_output))
    .route("/jobs/{job_id}/folder", post(open_output_folder))
    .route("/jobs/{job_id}/open", post(open_output))
    .route("/settings", get(get_settings).put(save_settings))
    .route("/settings/pick-download-directory", post(pick_download_directory))
    .route("/cache/clear", post(clear_cache))
    .with_state(state);

  #[cfg(dev)]
  {
    Router::new().nest("/api", api)
  }

  #[cfg(not(dev))]
  {
    Router::new()
      .nest("/api", api)
      .fallback(get(serve_embedded_asset))
  }
}

fn map_error(error: String) -> ApiError {
  ApiError { message: error }
}

async fn analyze(Json(payload): Json<AnalyzeRequest>) -> Result<Json<crate::models::VideoAnalysis>, ApiError> {
  commands::analyze_youtube_url_impl(payload.url)
    .map(Json)
    .map_err(map_error)
}

async fn preview(Json(payload): Json<PreviewRequest>) -> Result<Json<crate::models::PreviewSource>, ApiError> {
  commands::get_preview_source_impl(payload.source)
    .map(Json)
    .map_err(map_error)
}

async fn create_job(
  State(state): State<SharedState>,
  Json(request): Json<ClipRequest>,
) -> Result<Json<crate::models::DownloadJob>, ApiError> {
  commands::create_clip_job_impl(request, &state)
    .map(Json)
    .map_err(map_error)
}

async fn list_jobs(State(state): State<SharedState>) -> Result<Json<Vec<crate::models::DownloadJob>>, ApiError> {
  commands::list_jobs_impl(&state).map(Json).map_err(map_error)
}

async fn get_job(
  State(state): State<SharedState>,
  Path(job_id): Path<String>,
) -> Result<Json<crate::models::DownloadJob>, ApiError> {
  commands::get_job_impl(job_id, &state).map(Json).map_err(map_error)
}

async fn cancel_job(
  State(state): State<SharedState>,
  Path(job_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  commands::cancel_job_impl(job_id, &state)
    .map(|_| StatusCode::NO_CONTENT)
    .map_err(map_error)
}

async fn remove_job(
  State(state): State<SharedState>,
  Path(job_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  commands::remove_job_impl(job_id, &state)
    .map(|_| StatusCode::NO_CONTENT)
    .map_err(map_error)
}

async fn open_output(
  State(state): State<SharedState>,
  Path(job_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  commands::open_output_impl(job_id, &state)
    .map(|_| StatusCode::NO_CONTENT)
    .map_err(map_error)
}

async fn play_output(
  State(state): State<SharedState>,
  Path(job_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  commands::play_output_impl(job_id, &state)
    .map(|_| StatusCode::NO_CONTENT)
    .map_err(map_error)
}

async fn open_output_folder(
  State(state): State<SharedState>,
  Path(job_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  commands::open_output_folder_impl(job_id, &state)
    .map(|_| StatusCode::NO_CONTENT)
    .map_err(map_error)
}

async fn get_settings(State(state): State<SharedState>) -> Result<Json<AppSettings>, ApiError> {
  commands::get_settings_impl(&state).map(Json).map_err(map_error)
}

async fn save_settings(
  State(state): State<SharedState>,
  Json(settings): Json<AppSettings>,
) -> Result<Json<AppSettings>, ApiError> {
  commands::save_settings_impl(settings, &state)
    .map(Json)
    .map_err(map_error)
}

async fn pick_download_directory() -> Result<Json<Option<String>>, ApiError> {
  commands::pick_download_directory_impl()
    .map(Json)
    .map_err(map_error)
}

async fn clear_cache(State(state): State<SharedState>) -> Result<Json<u64>, ApiError> {
  commands::clear_cache_impl(&state).map(Json).map_err(map_error)
}

#[cfg(not(dev))]
async fn serve_embedded_asset(uri: Uri) -> Response {
  let request_path = match uri.path() {
    "/" => "index.html",
    path => path.trim_start_matches('/'),
  };

  let file = WEB_DIST
    .get_file(request_path)
    .or_else(|| WEB_DIST.get_file("index.html"));

  match file {
    Some(file) => {
      let mime = mime_guess::from_path(request_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
      (
        [(header::CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")))],
        file.contents(),
      )
        .into_response()
    }
    None => StatusCode::NOT_FOUND.into_response(),
  }
}
