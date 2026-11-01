import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  SystemInfo,
  DatabaseStats,
  RepositoryRecord,
  InstalledAppRecord,
  OperationRecord,
  CatalogPackageSummary,
  CatalogPackageDetails,
  RepositorySyncResult,
  WspManifest,
  InstallationResult,
  DownloadProgressPayload,
  UninstallResult,
  DetectedInstalledApp,
  AvailableUpdate,
} from '../types';

export const tauriService = {
  async getSystemInfo(): Promise<SystemInfo> {
    return await invoke<SystemInfo>('get_system_info');
  },

  async getDatabaseStatus(): Promise<DatabaseStats> {
    return await invoke<DatabaseStats>('get_database_status');
  },

  async getSettings(): Promise<Record<string, string>> {
    return await invoke<Record<string, string>>('get_settings');
  },

  async updateSetting(key: string, value: string): Promise<void> {
    return await invoke<void>('update_setting', { key, value });
  },

  async getRepositories(): Promise<RepositoryRecord[]> {
    return await invoke<RepositoryRecord[]>('get_repositories');
  },

  async addRepository(id: string, name: string, url: string, repoType?: string): Promise<void> {
    return await invoke<void>('add_repository', { id, name, url, repoType });
  },

  async getInstalledApplications(): Promise<InstalledAppRecord[]> {
    return await invoke<InstalledAppRecord[]>('get_installed_applications');
  },

  async getOperationsLog(): Promise<OperationRecord[]> {
    return await invoke<OperationRecord[]>('get_operations_log');
  },

  async searchCatalog(
    query?: string,
    category?: string,
    repositoryId?: string
  ): Promise<CatalogPackageSummary[]> {
    return await invoke<CatalogPackageSummary[]>('search_catalog', {
      query: query || null,
      category: category || null,
      repositoryId: repositoryId || null,
    });
  },

  async getPackageDetails(packageId: string): Promise<CatalogPackageDetails | null> {
    return await invoke<CatalogPackageDetails | null>('get_package_details', {
      packageId,
    });
  },

  async syncRepositories(): Promise<RepositorySyncResult[]> {
    return await invoke<RepositorySyncResult[]>('sync_repositories');
  },

  async validateManifestString(yamlContent: string): Promise<WspManifest> {
    return await invoke<WspManifest>('validate_manifest_string', {
      yamlContent,
    });
  },

  async installPackage(packageId: string, version?: string): Promise<InstallationResult> {
    return await invoke<InstallationResult>('install_package', {
      packageId,
      version: version || null,
    });
  },

  async listenDownloadProgress(
    callback: (payload: DownloadProgressPayload) => void
  ): Promise<UnlistenFn> {
    return await listen<DownloadProgressPayload>('download-progress', (event) => {
      callback(event.payload);
    });
  },

  async uninstallPackage(packageId: string): Promise<UninstallResult> {
    return await invoke<UninstallResult>('uninstall_package', { packageId });
  },

  async getDetectedInstalledApps(): Promise<DetectedInstalledApp[]> {
    return await invoke<DetectedInstalledApp[]>('get_detected_installed_apps');
  },

  async checkUpdates(): Promise<AvailableUpdate[]> {
    return await invoke<AvailableUpdate[]>('check_updates');
  },

  async toggleRepository(id: string, enabled: boolean): Promise<void> {
    return await invoke<void>('toggle_repository', { id, enabled });
  },

  async removeRepository(id: string): Promise<boolean> {
    return await invoke<boolean>('remove_repository', { id });
  },

  async updateRepositoryTrust(id: string, trustPolicy: string): Promise<void> {
    return await invoke<void>('update_repository_trust', { id, trustPolicy });
  },

  async verifySignature(publicKeyHex: string, message: string, signatureHex: string): Promise<boolean> {
    return await invoke<boolean>('verify_signature', {
      publicKeyHex,
      message,
      signatureHex,
    });
  },
};
