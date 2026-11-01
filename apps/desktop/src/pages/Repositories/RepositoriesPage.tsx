import React, { useState } from 'react';
import {
  Database,
  Plus,
  ShieldCheck,
  ShieldAlert,
  Globe,
  CheckCircle2,
  X,
  AlertTriangle,
  RefreshCw,
  Trash2,
  ToggleLeft,
  ToggleRight,
  Key,
} from 'lucide-react';
import {
  useRepositories,
  useAddRepository,
  useSyncRepositories,
  useToggleRepository,
  useRemoveRepository,
  useUpdateRepositoryTrust,
} from '../../hooks/useBackend';

export const RepositoriesPage: React.FC = () => {
  const { data: repos, isLoading, refetch } = useRepositories();
  const addRepoMutation = useAddRepository();
  const syncMutation = useSyncRepositories();
  const toggleRepoMutation = useToggleRepository();
  const removeRepoMutation = useRemoveRepository();
  const updateTrustMutation = useUpdateRepositoryTrust();

  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newRepoId, setNewRepoId] = useState('');
  const [newRepoName, setNewRepoName] = useState('');
  const [newRepoUrl, setNewRepoUrl] = useState('');
  const [notice, setNotice] = useState<string | null>(null);

  const handleAddRepo = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newRepoName.trim() || !newRepoUrl.trim()) return;

    const id = newRepoId.trim() || `community-${Date.now()}`;

    addRepoMutation.mutate(
      {
        id,
        name: newRepoName.trim(),
        url: newRepoUrl.trim(),
        repoType: 'community',
      },
      {
        onSuccess: () => {
          setNewRepoId('');
          setNewRepoName('');
          setNewRepoUrl('');
          setIsAddModalOpen(false);
          setNotice(`Community repository "${newRepoName.trim()}" added to local SQLite database.`);
          setTimeout(() => setNotice(null), 5000);
          refetch();
        },
        onError: (err) => {
          setNotice(`Failed to add repository: ${String(err)}`);
        },
      }
    );
  };

  const handleToggle = (id: string, currentEnabled: boolean) => {
    toggleRepoMutation.mutate(
      { id, enabled: !currentEnabled },
      {
        onSuccess: () => {
          setNotice(`Repository ${id} is now ${!currentEnabled ? 'enabled' : 'disabled'}.`);
          setTimeout(() => setNotice(null), 4000);
        },
      }
    );
  };

  const handleRemove = (id: string, name: string) => {
    if (!window.confirm(`Are you sure you want to remove repository "${name}" and all its cached packages?`)) {
      return;
    }
    removeRepoMutation.mutate(id, {
      onSuccess: () => {
        setNotice(`Repository "${name}" and its cached index were removed.`);
        setTimeout(() => setNotice(null), 4000);
      },
    });
  };

  const handleTrustChange = (id: string, newTrust: string) => {
    updateTrustMutation.mutate(
      { id, trustPolicy: newTrust },
      {
        onSuccess: () => {
          setNotice(`Repository trust policy updated to ${newTrust.toUpperCase()}.`);
          setTimeout(() => setNotice(null), 4000);
        },
      }
    );
  };

  const handleSyncAll = () => {
    syncMutation.mutate(undefined, {
      onSuccess: (results) => {
        const total = results.reduce((acc, r) => acc + r.packages_synced, 0);
        setNotice(`Synchronized all active repositories! Updated ${total} package(s).`);
        setTimeout(() => setNotice(null), 5000);
      },
    });
  };

  return (
    <div className="page-content">
      <div className="page-intro-with-action">
        <div>
          <h3>Repository Network Management</h3>
          <p className="page-subtext">
            T3 Hub supports a hybrid distribution architecture with official and community repositories,
            backed by Ed25519 cryptographic signature verification.
          </p>
        </div>
        <div style={{ display: 'flex', gap: 10 }}>
          <button
            onClick={handleSyncAll}
            disabled={syncMutation.isPending}
            className="btn btn-secondary"
          >
            <RefreshCw size={15} className={syncMutation.isPending ? 'spin' : ''} />
            {syncMutation.isPending ? 'Syncing...' : 'Sync All Repositories'}
          </button>
          <button onClick={() => setIsAddModalOpen(true)} className="btn btn-primary">
            <Plus size={16} /> Add Community Repository
          </button>
        </div>
      </div>

      {notice && (
        <div className="alert-banner alert-success">
          <CheckCircle2 size={18} />
          <span>{notice}</span>
        </div>
      )}

      {/* Loading state */}
      {isLoading && (
        <div className="empty-state">
          <div className="spinner" />
          <h4 style={{ marginTop: 16 }}>Loading configured repositories...</h4>
        </div>
      )}

      {/* Repositories Cards */}
      {!isLoading && repos && (
        <div className="repo-grid">
          {repos.map((repo) => {
            const isOfficial = repo.repo_type === 'official';
            return (
              <div key={repo.id} className={`repo-card ${!repo.enabled ? 'repo-card-disabled' : ''}`}>
                <div className="repo-card-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div className="repo-title-row">
                    <div className="card-icon-box">
                      <Database size={22} className={isOfficial ? 'text-accent' : 'text-primary'} />
                    </div>
                    <div>
                      <h4 className="repo-name">{repo.name}</h4>
                      <span className="repo-id font-mono text-xs">{repo.id}</span>
                    </div>
                  </div>

                  <div style={{ display: 'flex', gap: '6px' }}>
                    <button
                      onClick={() => handleToggle(repo.id, repo.enabled)}
                      className="btn btn-ghost btn-sm"
                      title={repo.enabled ? 'Disable repository' : 'Enable repository'}
                    >
                      {repo.enabled ? (
                        <ToggleRight size={20} className="text-success" />
                      ) : (
                        <ToggleLeft size={20} className="text-muted" />
                      )}
                    </button>
                    {!isOfficial && (
                      <button
                        onClick={() => handleRemove(repo.id, repo.name)}
                        className="btn btn-ghost btn-sm text-danger"
                        title="Remove community repository"
                      >
                        <Trash2 size={16} />
                      </button>
                    )}
                  </div>
                </div>

                <div className="repo-url-box">
                  <Globe size={14} className="text-muted" />
                  <span className="repo-url text-xs font-mono">{repo.url}</span>
                </div>

                <div className="repo-card-footer" style={{ marginTop: '12px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div className="repo-badges" style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                    {isOfficial ? (
                      <span className="badge badge-accent">
                        <ShieldCheck size={12} className="inline-icon" /> Official T3
                      </span>
                    ) : (
                      <span className="badge badge-outline">
                        <Globe size={12} className="inline-icon" /> Community
                      </span>
                    )}

                    {!isOfficial ? (
                      <select
                        value={repo.trust_policy}
                        onChange={(e) => handleTrustChange(repo.id, e.target.value)}
                        className="select-field text-xs"
                        style={{ padding: '2px 6px', height: '24px' }}
                      >
                        <option value="verified">Trust: VERIFIED</option>
                        <option value="user-approved">Trust: USER-APPROVED</option>
                        <option value="untrusted">Trust: UNTRUSTED</option>
                      </select>
                    ) : (
                      <span className="badge badge-success">
                        <Key size={11} className="inline-icon" /> Ed25519 Verified
                      </span>
                    )}
                  </div>

                  <span className="repo-status-indicator">
                    {repo.enabled ? (
                      <span className="text-success text-xs font-semibold">● Active</span>
                    ) : (
                      <span className="text-muted text-xs">○ Disabled</span>
                    )}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Security Architectural Notice */}
      <div className="architecture-note">
        <div className="note-header">
          <ShieldAlert size={18} className="text-accent" />
          <h5>Cryptographic Trust & Signature Model (Milestone 5)</h5>
        </div>
        <p className="note-text">
          Adding a community repository never grants it silent trust. All repository metadata is signed with Ed25519,
          transported over HTTPS, and verified against SHA-256 digests before package extraction or execution.
        </p>
      </div>

      {/* Add Repository Modal */}
      {isAddModalOpen && (
        <div className="modal-backdrop" onClick={() => setIsAddModalOpen(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3 className="modal-title">Add Community Repository</h3>
              <button
                onClick={() => setIsAddModalOpen(false)}
                className="modal-close-btn"
                title="Close"
              >
                <X size={20} />
              </button>
            </div>

            <form onSubmit={handleAddRepo}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Repository Identifier (ID)</label>
                  <input
                    type="text"
                    placeholder="e.g. org-tools or community-graphics"
                    value={newRepoId}
                    onChange={(e) => setNewRepoId(e.target.value)}
                    className="form-input font-mono"
                  />
                  <span className="form-hint">Unique slug without spaces.</span>
                </div>

                <div className="form-group">
                  <label className="form-label">Repository Display Name</label>
                  <input
                    type="text"
                    required
                    placeholder="e.g. My Organization Repository"
                    value={newRepoName}
                    onChange={(e) => setNewRepoName(e.target.value)}
                    className="form-input"
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">HTTPS Repository URL</label>
                  <input
                    type="text"
                    required
                    placeholder="https://example.com/repository/ or file:/// path"
                    value={newRepoUrl}
                    onChange={(e) => setNewRepoUrl(e.target.value)}
                    className="form-input"
                  />
                  <span className="form-hint">
                    Endpoint containing `repository.json`, `index.json`, and manifests.
                  </span>
                </div>

                <div className="alert-banner alert-warning" style={{ margin: '16px 0 0 0' }}>
                  <AlertTriangle size={18} />
                  <span>
                    Community repositories default to <strong>user-approved</strong> trust.
                    Packages will display cryptographic publisher indicators before installation.
                  </span>
                </div>
              </div>

              <div className="modal-footer">
                <button
                  type="button"
                  onClick={() => setIsAddModalOpen(false)}
                  className="btn btn-secondary"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={addRepoMutation.isPending}
                  className="btn btn-primary"
                >
                  <Plus size={15} /> {addRepoMutation.isPending ? 'Adding...' : 'Add Repository'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
