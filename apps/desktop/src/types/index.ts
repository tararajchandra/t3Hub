export type NavigationTab = 
  | 'home' 
  | 'discover' 
  | 'installed' 
  | 'updates' 
  | 'repositories' 
  | 'settings';

export type ThemeMode = 'dark' | 'light' | 'system';

export interface SystemInfo {
  os: string;
  arch: string;
  app_name: string;
  app_version: string;
  tauri_version: string;
}

export interface DatabaseStats {
  schema_version: number;
  db_path: string;
  repositories_count: number;
  packages_count: number;
  installations_count: number;
  operations_count: number;
}

export interface RepositoryRecord {
  id: string;
  name: string;
  url: string;
  repo_type: 'official' | 'community';
  enabled: boolean;
  trust_policy: 'verified' | 'user-approved' | 'untrusted';
  created_at: string;
  updated_at: string;
}

export interface InstalledAppRecord {
  id: string;
  package_id: string;
  name: string;
  version: string;
  publisher: string;
  install_scope: string;
  install_path: string | null;
  installed_at: string;
  status: string;
}

export interface OperationRecord {
  id: string;
  package_id: string;
  operation_type: string;
  target_version: string;
  status: string;
  logs: string | null;
  created_at: string;
  completed_at: string | null;
}

export interface CatalogPackageSummary {
  id: string;
  repository_id: string;
  repository_name: string;
  name: string;
  description: string | null;
  publisher: string;
  license: string | null;
  homepage: string | null;
  categories: string[];
  tags: string[];
  latest_version: string;
  icon_url: string | null;
  installed_version: string | null;
}

export interface PackageVersionDetail {
  version: string;
  release_channel: string;
  release_date: string | null;
  manifest_raw: string;
}

export interface CatalogPackageDetails {
  summary: CatalogPackageSummary;
  versions: PackageVersionDetail[];
}

export interface RepositorySyncResult {
  repository_id: string;
  packages_synced: number;
  success: boolean;
  error_message: string | null;
}

export interface WspManifest {
  schema_version: string;
  package: {
    id: string;
    name: string;
    version: string;
    description?: string;
    publisher: string;
    license?: string;
    homepage?: string;
    repository?: string;
    categories?: string[];
    tags?: string[];
  };
  platform: {
    os: string;
    architectures: string[];
    minimum_windows_version?: string;
  };
  release?: {
    channel: string;
    release_date?: string;
  };
  artifacts: Array<{
    id: string;
    type: 'exe' | 'msi' | 'zip';
    url: string;
    sha256: string;
    size?: number;
  }>;
  installation: {
    type: 'exe' | 'msi' | 'zip';
    artifact: string;
    scope: 'user' | 'machine';
    arguments?: string[];
    timeout_seconds?: number;
  };
  uninstallation?: {
    type: 'registry' | 'zip_directory' | 'msi';
    detect?: {
      registry_path?: string;
      directory_path?: string;
    };
  };
  updates?: {
    strategy: string;
    channel: string;
  };
}

export interface PackageCardItem {
  id: string;
  name: string;
  version: string;
  publisher: string;
  description: string;
  category: string;
  icon?: string;
  license: string;
  architecture: string[];
  installerType: 'exe' | 'msi' | 'zip';
  sha256?: string;
  sizeMb?: number;
  featured?: boolean;
  popular?: boolean;
  recentlyUpdated?: boolean;
  rating?: number;
}

export interface InstallationResult {
  success: boolean;
  install_path: string | null;
  logs: string;
  error_message: string | null;
}

export interface DownloadProgressPayload {
  package_id: string;
  bytes_downloaded: number;
  total_bytes: number | null;
  percentage: number;
}

export interface UninstallResult {
  success: boolean;
  error_message: string | null;
}

export interface DetectedInstalledApp {
  display_name: string;
  display_version: string | null;
  publisher: string | null;
  install_location: string | null;
  uninstall_string: string | null;
  quiet_uninstall_string: string | null;
  install_date: string | null;
  is_msi: boolean;
  registry_path: string;
  scope: string;
}

export interface AvailableUpdate {
  package_id: string;
  package_name: string;
  publisher: string;
  installed_version: string;
  new_version: string;
  size_mb: number | null;
  release_notes: string | null;
  channel: string;
  repository_id: string;
}
