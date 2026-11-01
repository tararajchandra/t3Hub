import React, { useState } from 'react';
import {
  X,
  ShieldCheck,
  Download,
  Copy,
  Check,
  FileCode,
  Layers,
  History,
  Info,
} from 'lucide-react';
import { usePackageDetails } from '../../hooks/useBackend';

interface PackageDetailModalProps {
  packageId: string;
  onClose: () => void;
  onInstall: (pkgName: string, version: string) => void;
}

export const PackageDetailModal: React.FC<PackageDetailModalProps> = ({
  packageId,
  onClose,
  onInstall,
}) => {
  const { data: details, isLoading } = usePackageDetails(packageId);
  const [activeTab, setActiveTab] = useState<'overview' | 'security' | 'manifest' | 'history'>('overview');
  const [copiedHash, setCopiedHash] = useState(false);

  if (isLoading) {
    return (
      <div className="modal-backdrop" onClick={onClose}>
        <div className="modal-content" onClick={(e) => e.stopPropagation()}>
          <div style={{ padding: 40, textAlign: 'center' }}>
            <div className="spinner" />
            <p style={{ marginTop: 16 }}>Loading package specifications...</p>
          </div>
        </div>
      </div>
    );
  }

  if (!details) {
    return (
      <div className="modal-backdrop" onClick={onClose}>
        <div className="modal-content" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h4>Package Not Found</h4>
            <button onClick={onClose} className="modal-close-btn"><X size={18} /></button>
          </div>
          <div className="modal-body">
            <p>Could not retrieve specifications for package ID: <code>{packageId}</code></p>
          </div>
        </div>
      </div>
    );
  }

  const { summary, versions } = details;
  const latestManifestRaw = versions.length > 0 ? versions[0].manifest_raw : '';

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedHash(true);
    setTimeout(() => setCopiedHash(false), 2000);
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-content modal-lg" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="modal-header">
          <div className="modal-title-group">
            <div className="card-icon-box">
              <Download size={26} className="text-accent" />
            </div>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <h3 className="modal-title" style={{ margin: 0 }}>{summary.name}</h3>
                <span className="badge badge-subtle">{summary.repository_name}</span>
                {summary.installed_version && (
                  <span className="badge badge-success">Installed: v{summary.installed_version}</span>
                )}
              </div>
              <p className="modal-subtitle">
                Published by <strong>{summary.publisher}</strong> • Latest: v{summary.latest_version}
              </p>
            </div>
          </div>
          <button onClick={onClose} className="modal-close-btn" title="Close dialog">
            <X size={20} />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="modal-tabs">
          <button
            className={`tab-btn ${activeTab === 'overview' ? 'tab-btn-active' : ''}`}
            onClick={() => setActiveTab('overview')}
          >
            <Info size={15} /> Overview
          </button>
          <button
            className={`tab-btn ${activeTab === 'security' ? 'tab-btn-active' : ''}`}
            onClick={() => setActiveTab('security')}
          >
            <ShieldCheck size={15} /> Integrity & Security
          </button>
          <button
            className={`tab-btn ${activeTab === 'manifest' ? 'tab-btn-active' : ''}`}
            onClick={() => setActiveTab('manifest')}
          >
            <FileCode size={15} /> Manifest (T3P)
          </button>
          <button
            className={`tab-btn ${activeTab === 'history' ? 'tab-btn-active' : ''}`}
            onClick={() => setActiveTab('history')}
          >
            <History size={15} /> Release History ({versions.length})
          </button>
        </div>

        {/* Tab Content */}
        <div className="modal-body">
          {activeTab === 'overview' && (
            <div>
              <p className="modal-desc">{summary.description || 'No description provided.'}</p>

              <div className="detail-section">
                <h5 className="detail-heading">Package Information</h5>
                <div className="spec-table">
                  <div className="spec-row">
                    <span className="spec-key">Package ID</span>
                    <span className="spec-val font-mono">{summary.id}</span>
                  </div>
                  <div className="spec-row">
                    <span className="spec-key">Publisher</span>
                    <span className="spec-val">{summary.publisher}</span>
                  </div>
                  <div className="spec-row">
                    <span className="spec-key">License</span>
                    <span className="spec-val">{summary.license || 'Proprietary / Custom'}</span>
                  </div>
                  <div className="spec-row">
                    <span className="spec-key">Categories</span>
                    <span className="spec-val">{summary.categories.join(', ') || 'General'}</span>
                  </div>
                  {summary.homepage && (
                    <div className="spec-row">
                      <span className="spec-key">Official Homepage</span>
                      <span className="spec-val">
                        <a href={summary.homepage} target="_blank" rel="noreferrer" className="text-accent">
                          {summary.homepage}
                        </a>
                      </span>
                    </div>
                  )}
                </div>
              </div>

              {summary.tags.length > 0 && (
                <div style={{ marginTop: 16 }}>
                  <h5 className="detail-heading">Tags</h5>
                  <div className="card-tags">
                    {summary.tags.map((t) => (
                      <span key={t} className="tag-pill">#{t}</span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'security' && (
            <div>
              <div className="security-box">
                <div className="security-header">
                  <ShieldCheck size={20} className="text-success" />
                  <span className="security-title">SHA-256 Digest & Repository Assurance</span>
                </div>
                <p className="security-text">
                  T3 Engine guarantees that downloaded packages cannot be tampered with in transit.
                  Every binary is validated against cryptographic signatures before being passed to installation adapters.
                </p>
              </div>

              <div className="detail-section" style={{ marginTop: 16 }}>
                <h5 className="detail-heading">Integrity Verification</h5>
                <div className="spec-table">
                  <div className="spec-row">
                    <span className="spec-key">Repository Origin</span>
                    <span className="spec-val">{summary.repository_name} ({summary.repository_id})</span>
                  </div>
                  <div className="spec-row">
                    <span className="spec-key">Install Target Scope</span>
                    <span className="spec-val">Per-user (non-elevated by default)</span>
                  </div>
                  <div className="spec-row">
                    <span className="spec-key">Command Injection Guard</span>
                    <span className="spec-val text-success">Enabled (arguments strictly tokenized)</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'manifest' && (
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                <span style={{ fontSize: '0.85rem', color: 'var(--color-text-muted)' }}>
                  Validated WSP / T3P v1.0 Manifest Schema
                </span>
                <button
                  onClick={() => copyToClipboard(latestManifestRaw)}
                  className="btn btn-secondary btn-sm"
                  style={{ gap: 4 }}
                >
                  {copiedHash ? <Check size={14} className="text-success" /> : <Copy size={14} />}
                  {copiedHash ? 'Copied' : 'Copy YAML'}
                </button>
              </div>
              <pre className="yaml-code-view">
                <code>{latestManifestRaw || '# No raw manifest stored'}</code>
              </pre>
            </div>
          )}

          {activeTab === 'history' && (
            <div>
              <div className="versions-list">
                {versions.map((ver) => (
                  <div key={ver.version} className="version-item">
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <Layers size={16} className="text-accent" />
                        <strong>v{ver.version}</strong>
                        <span className="badge badge-subtle">{ver.release_channel}</span>
                      </div>
                      <span style={{ fontSize: '0.8rem', color: 'var(--color-text-muted)' }}>
                        {ver.release_date || 'Release date not specified'}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Action Footer */}
        <div className="modal-footer">
          <button onClick={onClose} className="btn btn-secondary">
            Close
          </button>
          <button
            onClick={() => {
              onInstall(summary.name, summary.latest_version);
              onClose();
            }}
            className="btn btn-primary"
          >
            <Download size={16} />
            {summary.installed_version ? 'Reinstall / Change Version' : 'Install Application'}
          </button>
        </div>
      </div>
    </div>
  );
};
