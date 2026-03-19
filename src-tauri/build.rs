use std::{
  env,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  println!("cargo:rerun-if-env-changed=LOCALAPPDATA");
  stage_external_binary("yt-dlp");
  stage_external_binary("ffmpeg");
  tauri_build::build()
}

fn stage_external_binary(command: &str) {
  let target = env::var("TARGET").expect("TARGET not set");
  let source = resolve_binary_path(command)
    .unwrap_or_else(|| panic!("Failed to locate required dependency for bundling: {command}"));

  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
  let bin_dir = manifest_dir.join("bin");
  fs::create_dir_all(&bin_dir).expect("failed to create external bin directory");

  let destination = bin_dir.join(format!("{command}-{target}.exe"));
  fs::copy(&source, &destination)
    .unwrap_or_else(|error| panic!("failed to stage {command} from {}: {error}", source.display()));
  println!("cargo:rerun-if-changed={}", source.display());
}

fn resolve_binary_path(command: &str) -> Option<PathBuf> {
  resolve_from_where(command).or_else(|| resolve_from_winget_links(command))
}

fn resolve_from_where(command: &str) -> Option<PathBuf> {
  let output = Command::new("where.exe").arg(command).output().ok()?;
  if !output.status.success() {
    return None;
  }

  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(PathBuf::from)
    .find_map(|path| canonical_binary_path(&path))
}

fn resolve_from_winget_links(command: &str) -> Option<PathBuf> {
  let local_app_data = env::var("LOCALAPPDATA").ok()?;
  let candidate = Path::new(&local_app_data)
    .join("Microsoft")
    .join("WinGet")
    .join("Links")
    .join(format!("{command}.exe"));
  canonical_binary_path(&candidate)
}

fn canonical_binary_path(path: &Path) -> Option<PathBuf> {
  if !path.exists() {
    return None;
  }

  fs::canonicalize(path).ok().filter(|resolved| resolved.is_file())
}
