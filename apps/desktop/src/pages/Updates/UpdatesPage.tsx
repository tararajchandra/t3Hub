import React, { useState } from 'react';
import { RefreshCw, CheckCircle2, ArrowUpCircle, Sparkles, ShieldCheck, Loader2, AlertCircle } from 'lucide-react';
import { useAvailableUpdates } from '../../hooks/useBackend';
import { tauriService } from '../../services/tauri';
import { useQueryClient } from '@tanstack/react-query';

export const UpdatesPage: React.FC = () => {
  const queryClient = useQueryClient();
  const { data: updates, isLoading, isRefetching, refetch } = useAvailableUpdates();

  const [activeUpdatingId, setActiveUpdatingId] = useState<string | null>(null);
  const [updatingAll, setUpdatingAll] = useState(false);
  const [updateNotice, setUpdateNotice] = useState<{ type: 'success' | 'info' | 'error'; message: string } | null>(null);

  const handleUpdate = async (packageId: string, name: string, newVersion: string) => {
    setActiveUpdatingId(packageId);
    setUpdateNotice({
      type: 'info',
      message: `Updating ${name} to v${newVersion}... Downloading and verifying SHA-256 integrity.`,
    });

    try {
      const res = await tauriService.installPackage(packageId, newVersion);
      if (res.success) {
        setUpdateNotice({
          type: 'success',
          message: `Successfully updated ${name} to v${newVersion}!`,
        });
        queryClient.invalidateQueries({ queryKey: ['availableUpdates'] });
        queryClient.invalidateQueries({ queryKey: ['installedApps'] });
      } else {
        setUpdateNotice({
          type: 'error',
          message: res.error_message || `Failed to update ${name}.`,
        });
      }
    } catch (err) {
      setUpdateNotice({
        type: 'error',
        message: `Update error: ${err instanceof Error ? err.message : String(err)}`,
      });
    } finally {
      setActiveUpdatingId(null);
    }
  };

  const handleUpdateAll = async () => {
    if (!updates || updates.length === 0) return;
    setUpdatingAll(true);
    setUpdateNotice({
      type: 'info',
      message: `Sequentially updating ${updates.length} applications...`,
    });

    let successCount = 0;
    for (const item of updates) {
      try {
        const res = await tauriService.installPackage(item.package_id, item.new_version);
        if (res.success) {
          successCount++;
        }
      } catch (e) {
        console.error('Update failed for', item.package_name, e);
      }
    }

    setUpdatingAll(false);
    setUpdateNotice({
      type: 'success',
      message: `Batch update complete! Successfully updated ${successCount} of ${updates.length} packages.`,
    });
    queryClient.invalidateQueries({ queryKey: ['availableUpdates'] });
    queryClient.invalidateQueries({ queryKey: ['installedApps'] });
  };

  return (
    <div className="page-content">
      <div className="page-intro-with-action">
        <div>
          <h3>Software Updates</h3>
          <p className="page-subtext">
            Compare installed binaries against verified repository manifests via SemVer 2.0.
          </p>
        </div>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button
            onClick={() => refetch()}
            disabled={isLoading || isRefetching}
            className="btn btn-secondary"
          >
            <RefreshCw size={14} className={isRefetching ? 'spin' : ''} />
            {isRefetching ? 'Checking...' : 'Check for Updates'}
          </button>
          {updates && updates.length > 0 && (
            <button
              onClick={handleUpdateAll}
              disabled={updatingAll || activeUpdatingId !== null}
              className="btn btn-primary"
            >
              {updatingAll ? <Loader2 size={15} className="spin" /> : <RefreshCw size={15} />}
              Update All ({updates.length})
            </button>
          )}
        </div>
      </div>

      {updateNotice && (
        <div
          className={`alert-banner ${
            updateNotice.type === 'success'
              ? 'alert-success'
              : updateNotice.type === 'error'
              ? 'alert-danger'
              : 'alert-info'
          }`}
          style={{ marginBottom: '16px' }}
        >
          {updateNotice.type === 'success' ? (
            <CheckCircle2 size={18} />
          ) : updateNotice.type === 'error' ? (
            <AlertCircle size={18} />
          ) : (
            <Loader2 size={18} className="spin" />
          )}
          <span>{updateNotice.message}</span>
        </div>
      )}

      {isLoading ? (
        <div className="empty-state">
          <Loader2 size={36} className="spin text-accent" />
          <p>Analyzing installed packages and checking repository versions...</p>
        </div>
      ) : updates && updates.length > 0 ? (
        <div className="updates-list">
          {updates.map((item) => {
            const isUpdatingThis = activeUpdatingId === item.package_id;
            return (
              <div key={item.package_id} className="update-card">
                <div className="update-card-main">
                  <div className="update-card-header">
                    <div>
                      <h4 className="update-card-title">{item.package_name}</h4>
                      <div className="version-diff">
                        <span className="badge badge-subtle">Installed: v{item.installed_version}</span>
                        <span className="version-arrow">→</span>
                        <span className="badge badge-success">Latest: v{item.new_version}</span>
                        <span className="badge badge-outline">{item.channel}</span>
                      </div>
                    </div>
                    <button
                      onClick={() => handleUpdate(item.package_id, item.package_name, item.new_version)}
                      disabled={isUpdatingThis || updatingAll}
                      className="btn btn-primary btn-sm"
                    >
                      {isUpdatingThis ? (
                        <>
                          <Loader2 size={14} className="spin" /> Updating...
                        </>
                      ) : (
                        <>
                          <ArrowUpCircle size={14} /> Update
                        </>
                      )}
                    </button>
                  </div>

                  <div className="update-notes">
                    <div className="update-notes-title">
                      <Sparkles size={14} className="text-accent" /> Release Highlights
                    </div>
                    <p className="update-notes-text">
                      {item.release_notes || 'Official package update released. See package changelog for detailed fixes.'}
                    </p>
                  </div>

                  <div className="update-footer text-xs text-muted" style={{ display: 'flex', gap: '16px' }}>
                    <span>
                      <ShieldCheck size={13} className="text-success inline-icon" />
                      Repository: {item.repository_id}
                    </span>
                    {item.size_mb && (
                      <span>Download Size: ~{item.size_mb.toFixed(1)} MB</span>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="empty-state">
          <CheckCircle2 size={42} className="text-success" />
          <h4>Your Software is Fully Up to Date</h4>
          <p>All installed applications match the latest version published across your active repositories.</p>
          <button onClick={() => refetch()} className="btn btn-secondary btn-sm">
            <RefreshCw size={14} /> Re-scan Repositories
          </button>
        </div>
      )}
    </div>
  );
};
