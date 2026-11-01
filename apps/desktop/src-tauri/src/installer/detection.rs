use serde::{Deserialize, Serialize};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledAppInfo {
    pub display_name: String,
    pub display_version: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
    pub uninstall_string: Option<String>,
    pub quiet_uninstall_string: Option<String>,
    pub install_date: Option<String>,
    pub is_msi: bool,
    pub registry_path: String,
    pub scope: String, // "user" | "machine"
}

pub struct AppDetector;

impl AppDetector {
    /// Detect all installed applications registered in Windows Uninstall registry
    pub fn detect_all_installed_apps() -> Vec<InstalledAppInfo> {
        let mut apps = Vec::new();

        // 1. Per-User installs (HKCU)
        Self::scan_registry_root(
            HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "user",
            &mut apps,
        );

        // 2. Machine-wide 64-bit installs (HKLM)
        Self::scan_registry_root(
            HKEY_LOCAL_MACHINE,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "machine",
            &mut apps,
        );

        // 3. Machine-wide 32-bit installs on 64-bit OS (WOW6432Node)
        Self::scan_registry_root(
            HKEY_LOCAL_MACHINE,
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "machine",
            &mut apps,
        );

        apps
    }

    /// Scan a specific registry root and subpath
    fn scan_registry_root(
        root_hkey: winreg::HKEY,
        subpath: &str,
        scope: &str,
        collector: &mut Vec<InstalledAppInfo>,
    ) {
        let root = RegKey::predef(root_hkey);
        let uninstall_key = match root.open_subkey_with_flags(subpath, KEY_READ) {
            Ok(k) => k,
            Err(_) => return, // Key doesn't exist or no permission
        };

        for key_name in uninstall_key.enum_keys().map_while(Result::ok) {
            let app_key = match uninstall_key.open_subkey_with_flags(&key_name, KEY_READ) {
                Ok(k) => k,
                Err(_) => continue,
            };

            // DisplayName is required for a user-facing application
            let display_name: String = match app_key.get_value("DisplayName") {
                Ok(name) => name,
                Err(_) => continue, // Skip internal patches, updates without display name
            };

            // Optional standard fields
            let display_version: Option<String> = app_key.get_value("DisplayVersion").ok();
            let publisher: Option<String> = app_key.get_value("Publisher").ok();
            let install_location: Option<String> = app_key.get_value("InstallLocation").ok();
            let uninstall_string: Option<String> = app_key.get_value("UninstallString").ok();
            let quiet_uninstall_string: Option<String> = app_key.get_value("QuietUninstallString").ok();
            let install_date: Option<String> = app_key.get_value("InstallDate").ok();

            // WindowsInstaller dword indicates MSI package
            let is_msi = app_key
                .get_value::<u32, _>("WindowsInstaller")
                .map(|val| val == 1)
                .unwrap_or(false);

            let reg_full_path = format!("{}\\{}", subpath, key_name);

            collector.push(InstalledAppInfo {
                display_name,
                display_version,
                publisher,
                install_location,
                uninstall_string,
                quiet_uninstall_string,
                install_date,
                is_msi,
                registry_path: reg_full_path,
                scope: scope.to_string(),
            });
        }
    }

    /// Check if a specific registry path (e.g. from manifest detect.registry_path) exists
    pub fn find_by_registry_path(subpath: &str) -> Option<InstalledAppInfo> {
        // Try HKCU first
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(app_key) = hkcu.open_subkey_with_flags(subpath, KEY_READ) {
            if let Ok(name) = app_key.get_value::<String, _>("DisplayName") {
                return Some(InstalledAppInfo {
                    display_name: name,
                    display_version: app_key.get_value("DisplayVersion").ok(),
                    publisher: app_key.get_value("Publisher").ok(),
                    install_location: app_key.get_value("InstallLocation").ok(),
                    uninstall_string: app_key.get_value("UninstallString").ok(),
                    quiet_uninstall_string: app_key.get_value("QuietUninstallString").ok(),
                    install_date: app_key.get_value("InstallDate").ok(),
                    is_msi: app_key.get_value::<u32, _>("WindowsInstaller").map(|v| v == 1).unwrap_or(false),
                    registry_path: subpath.to_string(),
                    scope: "user".to_string(),
                });
            }
        }

        // Try HKLM
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(app_key) = hklm.open_subkey_with_flags(subpath, KEY_READ) {
            if let Ok(name) = app_key.get_value::<String, _>("DisplayName") {
                return Some(InstalledAppInfo {
                    display_name: name,
                    display_version: app_key.get_value("DisplayVersion").ok(),
                    publisher: app_key.get_value("Publisher").ok(),
                    install_location: app_key.get_value("InstallLocation").ok(),
                    uninstall_string: app_key.get_value("UninstallString").ok(),
                    quiet_uninstall_string: app_key.get_value("QuietUninstallString").ok(),
                    install_date: app_key.get_value("InstallDate").ok(),
                    is_msi: app_key.get_value::<u32, _>("WindowsInstaller").map(|v| v == 1).unwrap_or(false),
                    registry_path: subpath.to_string(),
                    scope: "machine".to_string(),
                });
            }
        }

        None
    }

    /// Search detected applications by case-insensitive name match
    pub fn find_by_name(name_query: &str) -> Option<InstalledAppInfo> {
        let lower = name_query.to_lowercase();
        let all = Self::detect_all_installed_apps();
        all.into_iter().find(|app| app.display_name.to_lowercase().contains(&lower))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_system_installed_apps() {
        // Scanning Windows registry should run without panicking and return standard installed software
        let apps = AppDetector::detect_all_installed_apps();
        // Almost all Windows systems have at least 1-2 items in registry (e.g. Edge, VS Code, Git, or OS components)
        println!("Detected {} registered applications on this machine", apps.len());
        for app in apps.iter().take(3) {
            assert!(!app.display_name.is_empty());
        }
    }

    #[test]
    fn test_find_by_non_existent_registry_path() {
        let res = AppDetector::find_by_registry_path("Software\\NonExistentVendor\\BogusAppKey123");
        assert!(res.is_none());
    }
}
