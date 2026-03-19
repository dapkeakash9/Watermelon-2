mod commands;
mod helpers;
mod models;
mod server;
mod state;

use helpers::open_in_browser;
use server::{browser_entry_url, configure_background_host, emit_background_ready, start_background_server};
use state::{schedule_jobs, SharedState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let shared_state = SharedState::new().expect("failed to initialize application state");
  let scheduler_state = shared_state.clone();
  let server_state = shared_state.clone();

  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      let launch_url = browser_entry_url();
      let _ = emit_background_ready(app, &launch_url);
      let _ = open_in_browser(&launch_url);
    }))
    .manage(shared_state)
    .setup(move |app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      schedule_jobs(scheduler_state.0.clone());
      start_background_server(server_state.clone())?;
      let launch_url = browser_entry_url();
      configure_background_host(app, launch_url.clone())?;
      emit_background_ready(&app.handle().clone(), &launch_url);
      open_in_browser(&launch_url).map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::analyze_youtube_url,
      commands::get_preview_source,
      commands::create_clip_job,
      commands::list_jobs,
      commands::get_job,
      commands::cancel_job,
      commands::remove_job,
      commands::open_output,
      commands::play_output,
      commands::open_output_folder,
      commands::get_settings,
      commands::save_settings,
      commands::pick_download_directory,
      commands::clear_cache,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
