import React, { useState } from 'react';
import { PackageCheck, Trash2, HardDrive, AlertCircle, CheckCircle2, Search, Monitor, ShieldCheck, Loader2 } from 'lucide-react';
import { useInstalledApps, useDetectedApps, useUninstallPackage } from '../../hooks/useBackend';
import { useAppStore } from '../../hooks/useAppStore';

export const InstalledPage: React.FC = () => {
  const { data: installedApps, isLoading: loadingInstalled } = useInstalledApps();
  const { data: detectedApps, isLoading: loadingDetected } = useDetectedApps();
  const uninstallMutation = useUninstallPackage();
  const { setActiveTab } = useAppStore();

  const [viewMode, setViewMode] = useState<'managed' | 'detected'>('managed');
  const [searchQuery, setSearchQuery] = useState('');
  const [actionNotice, setActionNotice] = useState<{ type: 'success' | 'error'; message: string } | null>(null);
  const [uninstallingId, setUninstallingId] = useState<string | null>(null);

  const handleUninstall = async (packageId: string, name: string) => {
    if (!window.confirm(`Are you sure you want to uninstall "${name}"?`)) {
      return;
    }

    setUninstallingId(packageId);
    setActionNotice(null);

    try {
      const res = await uninstallMutation.mutateAsync(packageId);
      if (res.success) {
        setActionNotice({
          type: 'success',
          message: `"${name}" was successfully uninstalled and removed from managed inventory.`,
        });
      } else {
        setActionNotice({
          type: 'error',
          message: res.error_message || `Failed to uninstall "${name}".`,
        });
      }
    } catch (err) {
      setActionNotice({
        type: 'error',
        message: `Uninstallation error: ${err instanceof Error ? err.message : String(err)}`,
      });
    } finally {
      setUninstallingId(null);
    }
  };

  const filteredDetected = (detectedApps || []).filter((app) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return (
      app.display_name.toLowerCase().includes(q) ||
      (app.publisher && app.publisher.toLowerCase().includes(q))
    );
  });

  return (
    <div className="page-content">
      {/* Header Info */}
      <div className="page-intro" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <h3>Installed & Detected Applications</h3>
          <p className="page-subtext">
            Manage packages installed through T3 Hub or inspect software discovered via Windows Registry detection.
          </p>
        </div>

        {/* View Mode Toggle */}
        <div style={{ display: 'flex', gap: '8px', background: 'var(--bg-card)', padding: '4px', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
          <button
            onClick={() => setViewMode('managed')}
            className={`btn btn-sm ${viewMode === 'managed' ? 'btn-primary' : 'btn-ghost'}`}
            style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
          >
            <PackageCheck size={15} />
            T3 Hub Managed ({installedApps?.length || 0})
          </button>
          <button
            onClick={() => setViewMode('detected')}
            className={`btn btn-sm ${viewMode === 'detected' ? 'btn-primary' : 'btn-ghost'}`}
            style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
          >
            <Monitor size={15} />
            Windows Detected ({detectedApps?.length || 0})
          </button>
        </div>
      </div>

      {actionNotice && (
        <div className={`alert-banner ${actionNotice.type === 'success' ? 'alert-success' : 'alert-danger'}`} style={{ marginBottom: '16px' }}>
          {actionNotice.type === 'success' ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
          <span>{actionNotice.message}</span>
        </div>
      )}

      {/* VIEW MODE: T3 Hub Managed */}
      {viewMode === 'managed' && (
        <>
          {loadingInstalled ? (
            <div className="empty-state">
              <Loader2 size={32} className="spin text-accent" />
              <p>Loading installed applications...</p>
            </div>
          ) : installedApps && installedApps.length > 0 ? (
            <div className="table-card">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Application Name</th>
                    <th>Version</th>
                    <th>Publisher</th>
                    <th>Scope</th>
                    <th>Location</th>
                    <th>Installed Date</th>
                    <th style={{ textAlign: 'right' }}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {installedApps.map((app) => (
                    <tr key={app.id}>
                      <td className="font-semibold text-primary">
                        <PackageCheck size={16} className="text-accent inline-icon" />
                        {app.name}
                      </td>
                      <td>
                        <span className="badge badge-subtle">v{app.version}</span>
                      </td>
                      <td>{app.publisher}</td>
                      <td>
                        <span className="badge badge-outline">{app.install_scope}</span>
                      </td>
                      <td className="font-mono text-xs text-muted">
                        {app.install_path || 'Per-User Application Data'}
                      </td>
                      <td className="text-muted text-xs">
                        {new Date(app.installed_at).toLocaleDateString()}
                      </td>
                      <td style={{ textAlign: 'right' }}>
                        <button
                          onClick={() => handleUninstall(app.package_id, app.name)}
                          disabled={uninstallingId === app.package_id}
                          className="btn btn-danger btn-sm"
                          title="Uninstall application safely"
                        >
                          {uninstallingId === app.package_id ? (
                            <>
                              <Loader2 size={13} className="spin" /> Uninstalling...
                            </>
                          ) : (
                            <>
                              <Trash2 size={13} /> Uninstall
                            </>
                          )}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="empty-state">
              <HardDrive size={40} className="text-muted" />
              <h4>No Managed Applications Installed</h4>
              <p>Applications installed via T3 Hub will appear here with one-click clean uninstallation.</p>
              <button onClick={() => setActiveTab('discover')} className="btn btn-primary btn-sm">
                Explore Software Catalog
              </button>
            </div>
          )}
        </>
      )}

      {/* VIEW MODE: Windows Registry Detected Applications */}
      {viewMode === 'detected' && (
        <>
          <div style={{ marginBottom: '16px', display: 'flex', gap: '12px', alignItems: 'center' }}>
            <div style={{ position: 'relative', flex: 1 }}>
              <Search size={16} style={{ position: 'absolute', left: '12px', top: '10px', color: 'var(--text-muted)' }} />
              <input
                type="text"
                placeholder="Search Windows detected software by name or publisher..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="input-field"
                style={{ paddingLeft: '36px', width: '100%' }}
              />
            </div>
          </div>

          {loadingDetected ? (
            <div className="empty-state">
              <Loader2 size={32} className="spin text-accent" />
              <p>Scanning Windows Registry (HKCU and HKLM Uninstall keys)...</p>
            </div>
          ) : filteredDetected.length > 0 ? (
            <div className="table-card" style={{ maxHeight: '600px', overflowY: 'auto' }}>
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Detected Application</th>
                    <th>Version</th>
                    <th>Publisher</th>
                    <th>Scope</th>
                    <th>Package Format</th>
                    <th>Registry Key Location</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredDetected.map((app, idx) => (
                    <tr key={`${app.registry_path}-${idx}`}>
                      <td className="font-semibold text-primary">
                        <Monitor size={15} className="inline-icon text-muted" />
                        {app.display_name}
                      </td>
                      <td>
                        {app.display_version ? (
                          <span className="badge badge-subtle">v{app.display_version}</span>
                        ) : (
                          <span className="text-muted text-xs">—</span>
                        )}
                      </td>
                      <td className="text-xs">{app.publisher || 'Unknown Publisher'}</td>
                      <td>
                        <span className={`badge ${app.scope === 'user' ? 'badge-subtle' : 'badge-outline'}`}>
                          {app.scope}
                        </span>
                      </td>
                      <td>
                        <span className="badge badge-subtle">
                          {app.is_msi ? 'MSI' : 'Native EXE'}
                        </span>
                      </td>
                      <td className="font-mono text-xs text-muted" title={app.registry_path}>
                        {app.registry_path.length > 50 ? `${app.registry_path.slice(0, 47)}...` : app.registry_path}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="empty-state">
              <ShieldCheck size={40} className="text-muted" />
              <h4>No Matching Applications Found</h4>
              <p>No installed software matched your search filter.</p>
            </div>
          )}
        </>
      )}
    </div>
  );
};
