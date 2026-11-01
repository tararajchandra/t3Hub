import React, { useState, useEffect } from 'react';
import {
  Search,
  Filter,
  Download,
  CheckCircle2,
  RefreshCw,
  AlertCircle,
  Package,
  ArrowRight,
  HardDrive,
} from 'lucide-react';
import { useAppStore } from '../../hooks/useAppStore';
import { useCatalogSearch, useSyncRepositories, useInstallPackage } from '../../hooks/useBackend';
import { tauriService } from '../../services/tauri';
import { PackageDetailModal } from '../../components/catalog/PackageDetailModal';
import type { CatalogPackageSummary } from '../../types';

const CATEGORIES = [
  'All',
  'Graphics',
  'Photography',
  'Developer Tools',
  'Utilities',
  'Audio',
  'Media',
  'Security',
];

interface InstallStatusState {
  packageId: string;
  name: string;
  percentage: number;
  stage: 'downloading' | 'verifying' | 'extracting' | 'completed' | 'failed';
  error?: string;
  installPath?: string;
}

export const DiscoverPage: React.FC = () => {
  const {
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    selectedPackageId,
    setSelectedPackageId,
    setActiveTab,
  } = useAppStore();

  const [installStatus, setInstallStatus] = useState<InstallStatusState | null>(null);
  const [successBanner, setSuccessBanner] = useState<string | null>(null);

  // Live database catalog search via TanStack Query
  const { data: packages, isLoading, isError, refetch } = useCatalogSearch(
    searchQuery || undefined,
    selectedCategory === 'All' ? undefined : selectedCategory
  );

  const syncMutation = useSyncRepositories();
  const installMutation = useInstallPackage();

  // Listen to download progress events emitted by Tauri backend
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    tauriService
      .listenDownloadProgress((payload) => {
        setInstallStatus((prev) => {
          if (!prev || prev.packageId !== payload.package_id) return prev;
          const stage = payload.percentage >= 100 ? 'verifying' : 'downloading';
          return {
            ...prev,
            percentage: Math.min(100, Math.round(payload.percentage)),
            stage,
          };
        });
      })
      .then((unlistenFn) => {
        unlisten = unlistenFn;
      })
      .catch((err) => {
        console.error('Failed to attach download progress listener:', err);
      });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleInstallClick = (pkgId: string, pkgName: string, version: string) => {
    setInstallStatus({
      packageId: pkgId,
      name: pkgName,
      percentage: 0,
      stage: 'downloading',
    });

    installMutation.mutate(
      { packageId: pkgId, version },
      {
        onSuccess: (result) => {
          if (result.success) {
            setInstallStatus({
              packageId: pkgId,
              name: pkgName,
              percentage: 100,
              stage: 'completed',
              installPath: result.install_path || undefined,
            });
            setSuccessBanner(`Successfully installed ${pkgName} v${version}!`);
            setTimeout(() => setSuccessBanner(null), 6000);
          } else {
            setInstallStatus({
              packageId: pkgId,
              name: pkgName,
              percentage: 0,
              stage: 'failed',
              error: result.error_message || 'Installation encountered an error.',
            });
          }
        },
        onError: (err) => {
          setInstallStatus({
            packageId: pkgId,
            name: pkgName,
            percentage: 0,
            stage: 'failed',
            error: String(err),
          });
        },
      }
    );
  };

  const handleSyncClick = () => {
    syncMutation.mutate(undefined, {
      onSuccess: (results) => {
        const total = results.reduce((acc, r) => acc + r.packages_synced, 0);
        setSuccessBanner(`Repository synchronized! ${total} package(s) updated.`);
        setTimeout(() => setSuccessBanner(null), 5000);
      },
    });
  };

  return (
    <div className="page-content">
      {/* Category Pills & Action Bar */}
      <div className="filter-bar">
        <div className="category-pills">
          {CATEGORIES.map((cat) => (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={`pill-btn ${selectedCategory === cat ? 'pill-btn-active' : ''}`}
            >
              {cat}
            </button>
          ))}
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div className="results-count">
            <Filter size={14} style={{ display: 'inline', marginRight: 4 }} />
            {packages ? `${packages.length} package${packages.length === 1 ? '' : 's'}` : 'Loading...'}
          </div>

          <button
            onClick={handleSyncClick}
            disabled={syncMutation.isPending}
            className="btn btn-secondary btn-sm"
            style={{ gap: 6 }}
            title="Sync latest packages from all enabled repositories"
          >
            <RefreshCw size={14} className={syncMutation.isPending ? 'spin' : ''} />
            {syncMutation.isPending ? 'Syncing...' : 'Sync Catalog'}
          </button>
        </div>
      </div>

      {/* Live Installation Progress Banner */}
      {installStatus && installStatus.stage !== 'completed' && installStatus.stage !== 'failed' && (
        <div className="alert-banner alert-info" style={{ flexDirection: 'column', alignItems: 'stretch', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 600 }}>
              <Download size={16} className="inline-icon spin" />
              Installing {installStatus.name}:{' '}
              {installStatus.stage === 'verifying' ? 'Verifying SHA-256 Digest & Extracting...' : 'Downloading Artifact...'}
            </span>
            <span className="font-mono text-xs font-semibold">{installStatus.percentage}%</span>
          </div>
          <div className="progress-bar-track">
            <div
              className="progress-bar-fill"
              style={{ width: `${Math.max(5, installStatus.percentage)}%` }}
            />
          </div>
        </div>
      )}

      {/* Installation Success Banner */}
      {installStatus && installStatus.stage === 'completed' && (
        <div className="alert-banner alert-success" style={{ justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <CheckCircle2 size={18} />
            <div>
              <strong>{installStatus.name} installed successfully!</strong>
              {installStatus.installPath && (
                <div className="text-xs font-mono" style={{ opacity: 0.9, marginTop: 2 }}>
                  Location: {installStatus.installPath}
                </div>
              )}
            </div>
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={() => setInstallStatus(null)} className="btn btn-secondary btn-sm">
              Dismiss
            </button>
            <button onClick={() => setActiveTab('installed')} className="btn btn-primary btn-sm">
              <HardDrive size={14} /> View in Installed <ArrowRight size={14} />
            </button>
          </div>
        </div>
      )}

      {/* Installation Error Banner */}
      {installStatus && installStatus.stage === 'failed' && (
        <div className="alert-banner alert-warning" style={{ justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <AlertCircle size={18} />
            <div>
              <strong>Installation of {installStatus.name} failed:</strong>
              <div className="text-xs" style={{ marginTop: 2 }}>{installStatus.error}</div>
            </div>
          </div>
          <button onClick={() => setInstallStatus(null)} className="btn btn-secondary btn-sm">
            Dismiss
          </button>
        </div>
      )}

      {/* Sync Success Banner */}
      {successBanner && (
        <div className="alert-banner alert-success">
          <CheckCircle2 size={18} />
          <span>{successBanner}</span>
        </div>
      )}

      {/* Sync Error Banner */}
      {syncMutation.isError && (
        <div className="alert-banner alert-warning">
          <AlertCircle size={18} />
          <span>Sync failed: {String(syncMutation.error)}</span>
        </div>
      )}

      {/* Loading state */}
      {isLoading && (
        <div className="empty-state">
          <div className="spinner" />
          <h4 style={{ marginTop: 16 }}>Querying T3 Package Catalog...</h4>
        </div>
      )}

      {/* Error state */}
      {isError && (
        <div className="empty-state">
          <AlertCircle size={40} className="text-warning" />
          <h4>Failed to load catalog from local database</h4>
          <p>Please check SQLite connection or click below to retry.</p>
          <button onClick={() => refetch()} className="btn btn-secondary btn-sm">
            Retry Search
          </button>
        </div>
      )}

      {/* Packages Grid */}
      {!isLoading && !isError && packages && packages.length > 0 && (
        <div className="packages-grid">
          {packages.map((pkg: CatalogPackageSummary) => {
            const isThisInstalling = installStatus?.packageId === pkg.id && installStatus?.stage === 'downloading';
            return (
              <div
                key={pkg.id}
                className="package-card"
                onClick={() => setSelectedPackageId(pkg.id)}
              >
                <div className="card-top">
                  <div className="card-icon-box">
                    <Package size={22} className="text-accent" />
                  </div>
                  <div className="card-badges">
                    {pkg.categories[0] && (
                      <span className="badge badge-subtle">{pkg.categories[0]}</span>
                    )}
                    {pkg.installed_version && (
                      <span className="badge badge-success">Installed</span>
                    )}
                  </div>
                </div>

                <h4 className="card-title">{pkg.name}</h4>
                <p className="card-publisher">By {pkg.publisher}</p>
                <p className="card-desc">{pkg.description || 'No description provided.'}</p>

                <div className="card-tags">
                  {pkg.license && <span className="tag-pill">{pkg.license}</span>}
                  {pkg.tags.slice(0, 2).map((tag) => (
                    <span key={tag} className="tag-pill">
                      #{tag}
                    </span>
                  ))}
                </div>

                <div className="card-footer">
                  <span className="card-version">v{pkg.latest_version}</span>
                  <button
                    disabled={isThisInstalling}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleInstallClick(pkg.id, pkg.name, pkg.latest_version);
                    }}
                    className={`btn ${pkg.installed_version ? 'btn-secondary' : 'btn-primary'} btn-sm`}
                  >
                    <Download size={14} className={isThisInstalling ? 'spin' : ''} />
                    {isThisInstalling
                      ? 'Installing...'
                      : pkg.installed_version
                      ? 'Reinstall'
                      : 'Install'}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Empty State */}
      {!isLoading && !isError && packages && packages.length === 0 && (
        <div className="empty-state">
          <Search size={40} className="text-muted" />
          <h4>No packages matched your filter</h4>
          <p>
            {searchQuery
              ? `No packages found for "${searchQuery}".`
              : 'The package catalog is currently empty. Click "Sync Catalog" to fetch verified packages from official repositories.'}
          </p>
          <div style={{ display: 'flex', gap: 10, marginTop: 12 }}>
            {searchQuery && (
              <button
                onClick={() => {
                  setSearchQuery('');
                  setSelectedCategory('All');
                }}
                className="btn btn-secondary btn-sm"
              >
                Reset Search
              </button>
            )}
            <button
              onClick={handleSyncClick}
              disabled={syncMutation.isPending}
              className="btn btn-primary btn-sm"
            >
              <RefreshCw size={14} className={syncMutation.isPending ? 'spin' : ''} />
              Sync Official Catalog
            </button>
          </div>
        </div>
      )}

      {/* Package Details Modal */}
      {selectedPackageId && (
        <PackageDetailModal
          packageId={selectedPackageId}
          onClose={() => setSelectedPackageId(null)}
          onInstall={(pkgName, version) => {
            handleInstallClick(selectedPackageId, pkgName, version);
          }}
        />
      )}
    </div>
  );
};
