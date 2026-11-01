import React from 'react';
import {
  Sparkles,
  TrendingUp,
  Clock,
  ShieldCheck,
  Package,
  Database,
  ArrowRight,
  CheckCircle2,
} from 'lucide-react';
import { useAppStore } from '../../hooks/useAppStore';
import { useDatabaseStatus, useRepositories, useInstalledApps, useCatalogSearch } from '../../hooks/useBackend';

export const HomePage: React.FC = () => {
  const { setActiveTab, setSelectedPackageId, setSearchQuery } = useAppStore();
  const { data: dbStats } = useDatabaseStatus();
  const { data: repos } = useRepositories();
  const { data: installed } = useInstalledApps();
  const { data: catalogPackages } = useCatalogSearch();

  const packages = catalogPackages || [];
  const featured = packages.slice(0, 3);
  const popular = packages.slice(0, 4);
  const recent = packages.slice(0, 4);

  const handleCardClick = (pkgId: string) => {
    setSelectedPackageId(pkgId);
    setActiveTab('discover');
  };

  return (
    <div className="page-content">
      {/* Hero Welcome Spotlight */}
      <section className="hero-banner">
        <div className="hero-content">
          <div className="hero-pill">
            <Sparkles size={14} className="text-accent" />
            <span>Open-Source Windows Distribution Ecosystem</span>
          </div>
          <h1 className="hero-title">Discover, Install, & Verify Native Windows Applications</h1>
          <p className="hero-subtitle">
            T3 Hub distributes open-source and commercial applications with cryptographic
            SHA-256 integrity verification, signed manifests, and native Windows installer orchestration.
          </p>
          <div className="hero-actions">
            <button
              onClick={() => {
                setSearchQuery('');
                setActiveTab('discover');
              }}
              className="btn btn-primary"
            >
              Browse Catalog <ArrowRight size={16} />
            </button>
            <button
              onClick={() => setActiveTab('repositories')}
              className="btn btn-secondary"
            >
              <Database size={16} /> Manage Repositories
            </button>
          </div>
        </div>

        {/* Quick Hub Stats Card */}
        <div className="hero-stats-card">
          <div className="stat-row">
            <div className="stat-label">Repository Status</div>
            <div className="stat-value text-success">
              <CheckCircle2 size={15} style={{ display: 'inline', marginRight: 4 }} />
              {repos?.length || 1} Configured
            </div>
          </div>
          <div className="stat-row">
            <div className="stat-label">Database Engine</div>
            <div className="stat-value">SQLite WAL Active</div>
          </div>
          <div className="stat-row">
            <div className="stat-label">Local Installed</div>
            <div className="stat-value text-accent">{installed?.length || 0} Apps Managed</div>
          </div>
          <div className="stat-row">
            <div className="stat-label">Security Standard</div>
            <div className="stat-value">
              <ShieldCheck size={15} className="text-accent" style={{ display: 'inline', marginRight: 4 }} />
              T3P v1.0 / SHA-256
            </div>
          </div>
        </div>
      </section>

      {/* Featured Applications Section */}
      <section className="section-container">
        <div className="section-header">
          <div className="section-title">
            <Sparkles size={18} className="text-accent" />
            <h3>Featured Applications</h3>
          </div>
          <button onClick={() => setActiveTab('discover')} className="section-link">
            View All ({packages.length})
          </button>
        </div>

        {featured.length > 0 ? (
          <div className="packages-grid">
            {featured.map((pkg) => (
              <div key={pkg.id} className="package-card" onClick={() => handleCardClick(pkg.id)}>
                <div className="card-top">
                  <div className="card-icon-box">
                    <Package size={24} className="text-accent" />
                  </div>
                  <div className="card-badges">
                    <span className="badge badge-accent">Featured</span>
                    {pkg.categories[0] && <span className="badge badge-subtle">{pkg.categories[0]}</span>}
                  </div>
                </div>
                <h4 className="card-title">{pkg.name}</h4>
                <p className="card-publisher">By {pkg.publisher}</p>
                <p className="card-desc">{pkg.description || 'Verified native application.'}</p>
                <div className="card-footer">
                  <span className="card-version">v{pkg.latest_version}</span>
                  <span className="badge badge-outline">{pkg.repository_name}</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="empty-state" style={{ padding: 28 }}>
            <p>No applications cached yet. Visit the <strong>Discover</strong> tab to synchronize the official repository.</p>
          </div>
        )}
      </section>

      {/* Grid: Popular & Recently Updated */}
      {packages.length > 0 && (
        <div className="two-column-grid">
          {/* Popular Applications */}
          <section className="section-container">
            <div className="section-header">
              <div className="section-title">
                <TrendingUp size={18} className="text-accent" />
                <h3>Popular in Network</h3>
              </div>
            </div>
            <div className="compact-list">
              {popular.map((pkg) => (
                <div
                  key={pkg.id}
                  className="compact-item"
                  onClick={() => handleCardClick(pkg.id)}
                >
                  <div className="compact-item-left">
                    <div className="compact-icon">
                      <Package size={18} className="text-accent" />
                    </div>
                    <div>
                      <h5 className="compact-title">{pkg.name}</h5>
                      <span className="compact-sub">{pkg.publisher} • v{pkg.latest_version}</span>
                    </div>
                  </div>
                  <div className="compact-item-right">
                    <span className="badge badge-outline">{pkg.categories[0] || 'App'}</span>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* Recently Updated */}
          <section className="section-container">
            <div className="section-header">
              <div className="section-title">
                <Clock size={18} className="text-accent" />
                <h3>Recently Updated</h3>
              </div>
            </div>
            <div className="compact-list">
              {recent.map((pkg) => (
                <div
                  key={pkg.id}
                  className="compact-item"
                  onClick={() => handleCardClick(pkg.id)}
                >
                  <div className="compact-item-left">
                    <div className="compact-icon">
                      <Package size={18} className="text-accent" />
                    </div>
                    <div>
                      <h5 className="compact-title">{pkg.name}</h5>
                      <span className="compact-sub">{pkg.publisher} • v{pkg.latest_version}</span>
                    </div>
                  </div>
                  <div className="compact-item-right">
                    <span className="badge badge-success">v{pkg.latest_version}</span>
                  </div>
                </div>
              ))}
            </div>
          </section>
        </div>
      )}

      {/* Network & Local SQLite Health Bar */}
      <section className="health-card">
        <div className="health-left">
          <Database size={20} className="text-accent" />
          <div>
            <h4 className="health-title">Local SQLite Data Store & Cache</h4>
            <p className="health-desc">
              {dbStats
                ? `Schema Version ${dbStats.schema_version} • Total Cached Packages: ${dbStats.packages_count} • Repositories: ${dbStats.repositories_count}`
                : 'Initializing SQLite local store...'}
            </p>
          </div>
        </div>
        <button onClick={() => setActiveTab('settings')} className="btn btn-secondary btn-sm">
          Inspect Diagnostics
        </button>
      </section>
    </div>
  );
};
