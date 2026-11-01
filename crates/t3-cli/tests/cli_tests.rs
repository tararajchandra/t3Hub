use std::fs;
use t3_cli::commands::index_generate::{execute_index_generate, IndexGenOptions};
use t3_cli::commands::init::{execute_init, InitOptions};
use t3_cli::commands::validate::execute_validate;
use t3_cli::crypto::ed25519::{derive_public_key, generate_keypair, sign_bytes, verify_bytes};
use t3_cli::crypto::hashing::compute_file_sha256;
use t3_cli::manifest::models::WspManifest;
use t3_cli::manifest::validator::validate_manifest;

#[test]
fn test_ed25519_keygen_sign_verify_flow() {
    let (secret_key, public_key) = generate_keypair().expect("Keygen failed");
    assert_eq!(secret_key.len(), 64);
    assert_eq!(public_key.len(), 64);

    let derived_pub = derive_public_key(&secret_key).expect("Derivation failed");
    assert_eq!(derived_pub, public_key);

    let message = b"T3 Hub Package Manifest Integrity Payload";
    let signature = sign_bytes(&secret_key, message).expect("Signing failed");
    assert_eq!(signature.len(), 128);

    // Verify valid signature
    let verify_result = verify_bytes(&public_key, message, &signature);
    assert!(verify_result.is_ok(), "Signature must verify successfully");

    // Tampered payload must fail
    let tampered = b"Tampered Package Payload";
    let tampered_result = verify_bytes(&public_key, tampered, &signature);
    assert!(tampered_result.is_err(), "Tampered message must be rejected");
}

#[test]
fn test_compute_file_sha256() {
    let temp_dir = std::env::temp_dir().join("t3_cli_test_hash");
    let _ = fs::create_dir_all(&temp_dir);
    let test_file = temp_dir.join("test_artifact.bin");

    // "hello world\n" known SHA-256
    fs::write(&test_file, b"hello world\n").expect("write failed");

    let (hash, size) = compute_file_sha256(&test_file).expect("hashing failed");
    assert_eq!(size, 12);
    assert_eq!(
        hash,
        "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447"
    );

    let _ = fs::remove_file(test_file);
}

#[test]
fn test_manifest_validation_pass() {
    let yaml = r#"
schema_version: "1.0"
package:
  id: "tools.calc"
  name: "Modern Calculator"
  version: "2.1.0"
  publisher: "MathCorp"
platform:
  os: "windows"
  architectures:
    - "x64"
artifacts:
  - id: "calc-x64-exe"
    type: "exe"
    url: "https://example.com/calc-setup.exe"
    sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
installation:
  type: "exe"
  artifact: "calc-x64-exe"
  scope: "user"
"#;

    let manifest: WspManifest = serde_yaml_ng::from_str(yaml).expect("YAML parse failed");
    let result = validate_manifest(&manifest);
    assert!(result.is_ok(), "Valid manifest should pass validation: {:?}", result.err());
}

#[test]
fn test_manifest_validation_fail_invalid_id() {
    let yaml = r#"
schema_version: "1.0"
package:
  id: "invalid_id_without_dot"
  name: "Bad App"
  version: "1.0.0"
  publisher: "Test"
platform:
  os: "windows"
  architectures: ["x64"]
artifacts:
  - id: "art1"
    type: "exe"
    url: "https://example.com/app.exe"
    sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
installation:
  type: "exe"
  artifact: "art1"
  scope: "user"
"#;

    let manifest: WspManifest = serde_yaml_ng::from_str(yaml).expect("YAML parse failed");
    let result = validate_manifest(&manifest);
    assert!(result.is_err());
    let errs = result.unwrap_err();
    assert!(errs.iter().any(|e| e.contains("reverse-DNS style namespace")));
}

#[test]
fn test_manifest_validation_fail_prohibited_shell_operators() {
    let yaml = r#"
schema_version: "1.0"
package:
  id: "security.exploit"
  name: "Exploit Test"
  version: "1.0.0"
  publisher: "Attacker"
platform:
  os: "windows"
  architectures: ["x64"]
artifacts:
  - id: "art1"
    type: "exe"
    url: "https://example.com/app.exe"
    sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
installation:
  type: "exe"
  artifact: "art1"
  scope: "user"
  arguments:
    - "/S"
    - "&& calc.exe"
"#;

    let manifest: WspManifest = serde_yaml_ng::from_str(yaml).expect("YAML parse failed");
    let result = validate_manifest(&manifest);
    assert!(result.is_err());
    let errs = result.unwrap_err();
    assert!(errs.iter().any(|e| e.contains("prohibited shell control characters")));
}

#[test]
fn test_init_scaffold_produces_valid_manifest() {
    let temp_dir = std::env::temp_dir().join("t3_cli_test_init");
    let _ = fs::create_dir_all(&temp_dir);
    let output_file = temp_dir.join("init_manifest.yaml");
    let _ = fs::remove_file(&output_file);

    execute_init(InitOptions {
        id: Some("demo.sampleapp".to_string()),
        name: Some("SampleApp".to_string()),
        version: Some("1.5.0".to_string()),
        publisher: Some("Sample Corp".to_string()),
        install_type: Some("exe".to_string()),
        scope: Some("user".to_string()),
        url: Some("https://example.com/dl/sample.exe".to_string()),
        sha256: Some("aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899".to_string()),
        output: output_file.to_string_lossy().to_string(),
    })
    .expect("execute_init failed");

    // Validate the generated manifest
    let validate_res = execute_validate(&output_file.to_string_lossy());
    assert!(validate_res.is_ok(), "Scaffolded manifest must be valid");

    let _ = fs::remove_file(output_file);
}

#[test]
fn test_index_generate_scans_manifests() {
    let temp_dir = std::env::temp_dir().join("t3_cli_test_repo");
    let _ = fs::create_dir_all(temp_dir.join("manifests/tools/myutil"));

    let v1 = r#"
schema_version: "1.0"
package:
  id: "tools.myutil"
  name: "MyUtil"
  version: "1.0.0"
  publisher: "ToolDev"
platform:
  os: "windows"
  architectures: ["x64"]
artifacts:
  - id: "art1"
    type: "exe"
    url: "https://example.com/v1.exe"
    sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
installation:
  type: "exe"
  artifact: "art1"
  scope: "user"
"#;

    let v2 = r#"
schema_version: "1.0"
package:
  id: "tools.myutil"
  name: "MyUtil"
  version: "2.0.0"
  publisher: "ToolDev"
platform:
  os: "windows"
  architectures: ["x64"]
artifacts:
  - id: "art2"
    type: "exe"
    url: "https://example.com/v2.exe"
    sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
installation:
  type: "exe"
  artifact: "art2"
  scope: "user"
"#;

    fs::write(temp_dir.join("manifests/tools/myutil/1.0.0.yaml"), v1).unwrap();
    fs::write(temp_dir.join("manifests/tools/myutil/2.0.0.yaml"), v2).unwrap();

    let output_index = temp_dir.join("index.json");

    execute_index_generate(IndexGenOptions {
        repo_dir: temp_dir.to_string_lossy().to_string(),
        repo_id: Some("test-repo".to_string()),
        output: Some(output_index.to_string_lossy().to_string()),
        sign_key: None,
    })
    .expect("Index generate failed");

    assert!(output_index.exists());
    let index_content = fs::read_to_string(&output_index).unwrap();
    assert!(index_content.contains("\"latest_version\": \"2.0.0\""));
    assert!(index_content.contains("\"id\": \"tools.myutil\""));
    assert!(index_content.contains("\"version\": \"1.0.0\""));
    assert!(index_content.contains("\"version\": \"2.0.0\""));

    let _ = fs::remove_dir_all(temp_dir);
}
