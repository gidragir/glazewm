#[path = "../../resources/build_support/winres.rs"]
mod winres;

fn main() {
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
    return;
  }

  println!("cargo:rerun-if-env-changed=VERSION_NUMBER");
  winres::compile_windows_resource(
    tauri_winres::WindowsResource::new(),
    "glazewm.exe",
    "GlazeWM CLI",
    "GlazeWM CLI",
  );
}
