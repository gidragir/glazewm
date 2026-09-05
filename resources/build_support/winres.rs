use tauri_winres::{VersionInfo, WindowsResource};

#[allow(dead_code)]
pub fn configure_windows_resource(
  res: &mut WindowsResource,
  filename: &str,
  product_name: &str,
  file_description: &str,
) {
  res.set_icon("../../resources/assets/icon.ico");

  // Set language to English (US).
  res.set_language(0x0409);

  res.set("OriginalFilename", filename);
  res.set("ProductName", product_name);
  res.set("FileDescription", file_description);

  let version_env = option_env!("VERSION_NUMBER").unwrap_or("0.0.0");
  let version_parts = version_env
    .split('.')
    .take(3)
    .map(|part| part.parse().unwrap_or(0))
    .collect::<Vec<u16>>();

  let [major, minor, patch] =
    <[u16; 3]>::try_from(version_parts).unwrap_or([0, 0, 0]);

  let version_str = format!("{major}.{minor}.{patch}.0");
  res.set("FileVersion", &version_str);
  res.set("ProductVersion", &version_str);

  let version_u64 = (u64::from(major) << 48)
    | (u64::from(minor) << 32)
    | (u64::from(patch) << 16);

  res.set_version_info(VersionInfo::FILEVERSION, version_u64);
  res.set_version_info(VersionInfo::PRODUCTVERSION, version_u64);
}

#[allow(dead_code)]
pub fn compile_windows_resource(
  mut res: WindowsResource,
  filename: &str,
  product_name: &str,
  file_description: &str,
) {
  configure_windows_resource(&mut res, filename, product_name, file_description);

  if let Err(err) = res.compile() {
    eprintln!("cargo:warning=Failed to compile Windows resource: {err}");
  }
}
