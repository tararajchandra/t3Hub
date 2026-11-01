use chrono::Utc;
use std::fs;
use std::path::Path;

pub struct InitOptions {
    pub id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub install_type: Option<String>,
    pub scope: Option<String>,
    pub url: Option<String>,
    pub sha256: Option<String>,
    pub output: String,
}

pub fn execute_init(opts: InitOptions) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new(&opts.output);
    if output_path.exists() {
        return Err(format!(
            "Target file '{}' already exists. Choose another output or remove existing file.",
            opts.output
        )
        .into());
    }

    let id = opts.id.unwrap_or_else(|| "developer.myapp".to_string());
    let name = opts.name.unwrap_or_else(|| "MyApp".to_string());
    let version = opts.version.unwrap_or_else(|| "1.0.0".to_string());
    let publisher = opts.publisher.unwrap_or_else(|| "Acme Tools".to_string());
    let install_type = opts.install_type.unwrap_or_else(|| "exe".to_string());
    let scope = opts.scope.unwrap_or_else(|| "user".to_string());
    let url = opts.url.unwrap_or_else(|| {
        format!("https://example.com/downloads/{}-Setup-{}.exe", name, version)
    });
    let sha256 = opts.sha256.unwrap_or_else(|| {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    });
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let sample_args = match install_type.as_str() {
        "msi" => "    - \"/qn\"\n    - \"/norestart\"",
        "zip" => "    # Portable ZIP requires no installer arguments",
        _ => "    - \"/VERYSILENT\"\n    - \"/SUPPRESSMSGBOXES\"\n    - \"/NORESTART\"",
    };

    let uninst_block = match install_type.as_str() {
        "zip" => format!(
            "uninstallation:\n  type: \"zip_directory\"\n  detect:\n    directory_path: \"%LOCALAPPDATA%\\\\T3Hub\\\\Apps\\\\{}\"",
            id
        ),
        "msi" => "uninstallation:\n  type: \"msi\"".to_string(),
        _ => format!(
            "uninstallation:\n  type: \"registry\"\n  detect:\n    registry_path: \"Software\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Uninstall\\\\{}\"",
            name
        ),
    };

    let content = format!(
        r#"schema_version: "1.0"

package:
  id: "{id}"
  name: "{name}"
  version: "{version}"
  description: "Modern high-performance desktop tool for Windows."
  publisher: "{publisher}"
  license: "MIT"
  homepage: "https://example.com"
  repository: "https://github.com/developer/myapp"
  categories:
    - Utilities
    - Developer Tools
  tags:
    - windows
    - desktop
    - utility

platform:
  os: "windows"
  architectures:
    - "x64"
  minimum_windows_version: "10"

release:
  channel: "stable"
  release_date: "{today}"

artifacts:
  - id: "windows-x64-artifact"
    type: "{install_type}"
    url: "{url}"
    sha256: "{sha256}"
    # Optional size in bytes:
    # size: 25480000

installation:
  type: "{install_type}"
  artifact: "windows-x64-artifact"
  scope: "{scope}"
  arguments:
{sample_args}
  timeout_seconds: 600

{uninst_block}

updates:
  strategy: "version"
  channel: "stable"
"#
    );

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(output_path, content)?;

    println!("Created new T3 Hub manifest at: {}", opts.output);
    println!("Next steps:");
    println!("  1. Update URLs and metadata in '{}'", opts.output);
    println!("  2. Run 't3 digest <installer-file>' to calculate SHA-256");
    println!("  3. Validate using 't3 validate {}'", opts.output);

    Ok(())
}
