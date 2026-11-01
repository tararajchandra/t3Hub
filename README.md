# T3 Hub

> Open-Source Windows Software Distribution Ecosystem & Package Hub

T3 Hub (CLI: `t3`, Package Spec: `T3P v1.0`) is a high-performance Windows desktop package management platform and developer ecosystem for Windows 10 and 11. It provides seamless discovery, verified installation, background updating, and clean uninstallation of native Windows applications (EXE, MSI, and ZIP portable) across official and decentralized community repositories.

---

## 🌟 Core Features & Architecture Highlights

- **Desktop Client Shell:** Built on **Tauri 2** (Rust + Microsoft WebView2) for ultra-low memory footprint and instant startup.
- **Modern UI:** **React 19**, TypeScript, Vite, Zustand, TanStack Query, Lucide React with native dark/light themes.
- **Embedded Persistence:** Embedded **SQLite** in WAL mode with foreign key enforcement and automated versioned migrations.
- **Zero Antivirus / SmartScreen False Positives by Design:**
  - **No Arbitrary Scripts:** Declarative package manifests (`T3P v1.0`) eliminate untrusted shell/batch scripts.
  - **SHA-256 Checksum Pinning:** All artifacts are streamed and verified against pre-computed cryptographic digests before execution.
  - **Ed25519 Repository Signatures:** Asymmetric detached signatures protect repository indices against tampering.
  - **Per-User Isolation Defaults:** Standard installations target `%LOCALAPPDATA%`, preventing unnecessary UAC elevation prompts.
- **Modular Installer Engine:** Dedicated adapters for:
  - **MSI:** Silent invocation (`msiexec.exe /i <path> /qn /norestart`), exit-code interpretation (reboot required 3010 vs success 0).
  - **EXE:** Automated detection and profiles for Inno Setup, NSIS, and WiX installers without shell expansion.
  - **ZIP Portable:** Atomic extraction with strict Zip Slip / Path Traversal mitigation.
- **Application Detection & Uninstaller:** Scans `HKCU`, `HKLM` 64-bit, and `HKLM` WOW6432Node registry trees with isolated directory cleanup.
- **Developer CLI (`t3`):** Standalone Rust tool for authoring, linting, hashing, signing, and repository indexing.

---

## 📂 Monorepo Structure

```text
Installer/
├── Cargo.toml                 # Cargo workspace configuration
├── apps/
│   └── desktop/               # Tauri 2 Desktop Application
│       ├── src/               # React 19 UI (Home, Discover, Installed, Updates, Repos, Settings)
│       └── src-tauri/         # Rust backend (SQLite, manifest, installer, updater, security)
├── crates/
│   └── t3-cli/                # Developer CLI (`t3` binary)
│       ├── src/               # Commands: init, validate, digest, keygen, sign, verify, index-generate
│       └── tests/             # Comprehensive CLI integration test suite
├── repositories/
│   └── official/              # Official curated package repository
│       ├── repository.json    # Repository metadata and Ed25519 public key
│       ├── index.json         # Master package index
│       └── manifests/         # Category-organized YAML package manifests
└── docs/
    ├── architecture.md        # Complete system architecture specification
    └── publishing.md          # Developer packaging & publishing guide
```

---

## 🚀 Getting Started

### Prerequisites
- **Node.js:** $\ge$ 20.x
- **Rust Toolchain:** $\ge$ 1.90 (Rust 2024 edition)
- **C++ Build Tools:** MSVC & Windows 10/11 SDK

---

### Running the Desktop Application

#### 1. Frontend Development Server
```bash
cd apps/desktop
npm install
npm run dev
```

#### 2. Native Desktop Shell (Tauri 2)
```bash
cd apps/desktop
npx tauri dev
```

#### 3. Production Desktop Build
```bash
cd apps/desktop
npm run build
npx tauri build
```

---

### Using the Developer CLI (`t3`)

Build the standalone release binary:
```bash
cargo build --release --bin t3
```
The compiled executable is located at `target/release/t3.exe`.

#### CLI Commands Overview:

| Command | Description | Example |
|---|---|---|
| `t3 init` | Scaffolds a new compliant `manifest.yaml` | `t3 init --id "developer.mytool" --name "MyTool" --type exe` |
| `t3 digest` | Computes SHA-256 hash and byte size of an installer | `t3 digest "build/MyTool-Setup.exe"` |
| `t3 validate` | Validates manifests against `T3P v1.0` spec | `t3 validate "repositories/official/manifests"` |
| `t3 keygen` | Generates a new Ed25519 signing keypair | `t3 keygen --output "keys.json"` |
| `t3 sign` | Creates a detached Ed25519 signature for a file | `t3 sign "index.json" --key "<SECRET_KEY>"` |
| `t3 verify` | Verifies a detached Ed25519 signature | `t3 verify "index.json" --sig "index.json.sig" --public-key "<PUBKEY>"` |
| `t3 index-generate` | Scans manifests and compiles `index.json` | `t3 index-generate --repo-dir "repositories/official"` |

---

## 🧪 Testing

Run all unit and integration tests across the workspace:

```bash
# Test backend desktop library (32 tests)
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml

# Test developer CLI (6 tests)
cargo test --manifest-path crates/t3-cli/Cargo.toml

# Or run tests across the entire Cargo workspace
cargo test --workspace
```

---

## 📖 Documentation
- [Architecture Specification](docs/architecture.md)
- [Packaging & Publishing Guide](docs/publishing.md)

---

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🔒 Code Signing & Acknowledgements

Free code signing is provided by the [SignPath Foundation](https://about.signpath.io/open-source/).

Binary artifacts released on the [GitHub Releases](https://github.com/tararajchandra/t3Hub/releases) page are cryptographically signed using code signing certificates provided by the SignPath Foundation to ensure secure, tamper-proof distribution on Windows.
