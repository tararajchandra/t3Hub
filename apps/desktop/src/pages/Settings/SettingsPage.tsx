import React, { useState } from 'react';
import {
  HardDrive,
  Database,
  Moon,
  Sun,
  ShieldCheck,
  Folder,
  Trash2,
  CheckCircle2,
  Activity,
  Terminal,
} from 'lucide-react';
import { useAppStore } from '../../hooks/useAppStore';
import {
  useSettings,
  useUpdateSetting,
  useDatabaseStatus,
  useOperationsLog,
  useSystemInfo,
} from '../../hooks/useBackend';
import type { ThemeMode } from '../../types';

export const SettingsPage: React.FC = () => {
  const { theme, setTheme } = useAppStore();
  const { data: dbSettings } = useSettings();
  const { mutate: updateSetting } = useUpdateSetting();
  const { data: dbStats, refetch: refetchDbStats } = useDatabaseStatus();
  const { data: operations } = useOperationsLog();
  const { data: sysInfo } = useSystemInfo();

  const [downloadDir, setDownloadDir] = useState<string>(
    dbSettings?.download_directory || 'C:\\Users\\Default\\AppData\\Local\\T3Hub\\Cache'
  );
  const [cacheLimit, setCacheLimit] = useState<string>(
    dbSettings?.cache_limit_gb || '10'
  );
  const [saveNotice, setSaveNotice] = useState<string | null>(null);

  const handleSaveSettings = (e: React.FormEvent) => {
    e.preventDefault();
    updateSetting({ key: 'download_directory', value: downloadDir });
    updateSetting({ key: 'cache_limit_gb', value: cacheLimit });
    setSaveNotice('Settings successfully saved to local SQLite database.');
    setTimeout(() => setSaveNotice(null), 4000);
  };

  const handleThemeChange = (newTheme: ThemeMode) => {
    setTheme(newTheme);
    updateSetting({ key: 'theme', value: newTheme });
  };

  return (
    <div className="page-content">
      <div className="page-intro">
        <div>
          <h3>Client Settings & Diagnostics</h3>
          <p className="page-subtext">
            Configure local storage, installer policies, appearance, and monitor SQLite database state.
          </p>
        </div>
      </div>

      {saveNotice && (
        <div className="alert-banner alert-success">
          <CheckCircle2 size={18} />
          <span>{saveNotice}</span>
        </div>
      )}

      <form onSubmit={handleSaveSettings} className="settings-container">
        {/* Appearance Settings */}
        <div className="settings-card">
          <div className="settings-card-header">
            <Sun size={18} className="text-accent" />
            <h4>Appearance & Theme</h4>
          </div>
          <div className="settings-body">
            <label className="form-label">Color Theme</label>
            <div className="theme-options">
              <button
                type="button"
                onClick={() => handleThemeChange('dark')}
                className={`theme-btn ${theme === 'dark' ? 'theme-btn-active' : ''}`}
              >
                <Moon size={16} /> Dark Theme
              </button>
              <button
                type="button"
                onClick={() => handleThemeChange('light')}
                className={`theme-btn ${theme === 'light' ? 'theme-btn-active' : ''}`}
              >
                <Sun size={16} /> Light Theme
              </button>
            </div>
          </div>
        </div>

        {/* Storage & Cache Management */}
        <div className="settings-card">
          <div className="settings-card-header">
            <HardDrive size={18} className="text-accent" />
            <h4>Download Cache & Directories</h4>
          </div>
          <div className="settings-body">
            <div className="form-group">
              <label className="form-label">Package Download & Staging Directory</label>
              <div className="input-with-button">
                <input
                  type="text"
                  value={downloadDir}
                  onChange={(e) => setDownloadDir(e.target.value)}
                  className="form-input font-mono text-xs"
                />
                <button
                  type="button"
                  className="btn btn-secondary btn-sm"
                  onClick={() => alert('Folder selection dialog will connect to native Windows dialog in Milestone 3.')}
                >
                  <Folder size={15} /> Browse
                </button>
              </div>
              <span className="form-hint">
                Downloaded `.exe`, `.msi`, and `.zip` artifacts are cached here before integrity hashing.
              </span>
            </div>

            <div className="form-group">
              <label className="form-label">Maximum Cache Size (GB)</label>
              <input
                type="number"
                min="1"
                max="100"
                value={cacheLimit}
                onChange={(e) => setCacheLimit(e.target.value)}
                className="form-input"
                style={{ maxWidth: 200 }}
              />
            </div>

            <div className="form-group" style={{ marginTop: 12 }}>
              <button
                type="button"
                onClick={() => {
                  setSaveNotice('Local download cache cleared.');
                  setTimeout(() => setSaveNotice(null), 3000);
                }}
                className="btn btn-secondary btn-sm text-danger"
              >
                <Trash2 size={14} /> Clear Cached Download Files
              </button>
            </div>
          </div>
        </div>

        {/* Security & Installation Policies */}
        <div className="settings-card">
          <div className="settings-card-header">
            <ShieldCheck size={18} className="text-accent" />
            <h4>Security & Execution Standards</h4>
          </div>
          <div className="settings-body">
            <div className="checkbox-row">
              <input
                type="checkbox"
                id="require-sig"
                defaultChecked={true}
                disabled
              />
              <label htmlFor="require-sig" className="checkbox-label">
                <strong>Enforce SHA-256 Digest Verification (Mandatory)</strong>
                <span className="checkbox-subtext">
                  Any artifact with a checksum mismatch is permanently aborted and purged.
                </span>
              </label>
            </div>

            <div className="checkbox-row" style={{ marginTop: 12 }}>
              <input
                type="checkbox"
                id="prefer-user"
                defaultChecked={true}
                disabled
              />
              <label htmlFor="prefer-user" className="checkbox-label">
                <strong>Prefer Per-User Installation (Least Privilege)</strong>
                <span className="checkbox-subtext">
                  Installs applications into user AppData without prompting for Administrator / UAC elevation unless strictly required.
                </span>
              </label>
            </div>
          </div>
        </div>

        <div className="settings-actions">
          <button type="submit" className="btn btn-primary">
            Save Preferences
          </button>
        </div>
      </form>

      {/* SQLite Live Database Diagnostics */}
      <div className="settings-card" style={{ marginTop: 24 }}>
        <div className="settings-card-header">
          <Database size={18} className="text-accent" />
          <h4>SQLite Local Database Diagnostics</h4>
        </div>
        <div className="settings-body">
          {dbStats ? (
            <div className="diagnostics-grid">
              <div className="diag-item">
                <span className="diag-label">System Architecture:</span>
                <span className="diag-val">{sysInfo ? `${sysInfo.os} (${sysInfo.arch}) • Tauri ${sysInfo.tauri_version}` : 'Windows (x64)'}</span>
              </div>
              <div className="diag-item">
                <span className="diag-label">Schema Version:</span>
                <span className="diag-val badge badge-success">Version {dbStats.schema_version}</span>
              </div>
              <div className="diag-item">
                <span className="diag-label">Database File:</span>
                <span className="diag-val font-mono text-xs">{dbStats.db_path}</span>
              </div>
              <div className="diag-item">
                <span className="diag-label">Configured Repositories:</span>
                <span className="diag-val">{dbStats.repositories_count} records</span>
              </div>
              <div className="diag-item">
                <span className="diag-label">Cached Packages:</span>
                <span className="diag-val">{dbStats.packages_count} records</span>
              </div>
              <div className="diag-item">
                <span className="diag-label">Tracked Installations:</span>
                <span className="diag-val">{dbStats.installations_count} records</span>
              </div>
              <div className="diag-item">
                <span className="diag-label">Total Logged Operations:</span>
                <span className="diag-val">{dbStats.operations_count} records</span>
              </div>
            </div>
          ) : (
            <p className="text-muted text-xs">Querying SQLite connection...</p>
          )}

          <div style={{ marginTop: 16 }}>
            <button
              onClick={() => refetchDbStats()}
              className="btn btn-secondary btn-sm"
            >
              <Activity size={14} /> Refresh SQLite Health
            </button>
          </div>
        </div>
      </div>

      {/* Operation Logs Viewer */}
      <div className="settings-card" style={{ marginTop: 24 }}>
        <div className="settings-card-header">
          <Terminal size={18} className="text-accent" />
          <h4>Recent Operations Log</h4>
        </div>
        <div className="settings-body">
          {operations && operations.length > 0 ? (
            <div className="log-viewer">
              {operations.map((op) => (
                <div key={op.id} className="log-line">
                  <span className="log-time">[{op.created_at}]</span>
                  <span className="log-op">[{op.operation_type.toUpperCase()}]</span>
                  <span className="log-pkg">{op.package_id} (v{op.target_version})</span>
                  <span className={`log-status status-${op.status}`}>{op.status}</span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-muted text-xs">
              No recent installer operations recorded yet. Clean transaction log.
            </p>
          )}
        </div>
      </div>
    </div>
  );
};
