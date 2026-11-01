import React, { useEffect } from 'react';
import {
  Home,
  Compass,
  PackageCheck,
  RefreshCw,
  Database,
  Settings,
  HardDrive,
} from 'lucide-react';
import { useAppStore } from '../../hooks/useAppStore';
import { useDatabaseStatus, useSystemInfo } from '../../hooks/useBackend';
import type { NavigationTab } from '../../types';

interface NavItem {
  id: NavigationTab;
  label: string;
  icon: React.ElementType;
  keyHint: string;
  badge?: number | string;
}

const navItems: NavItem[] = [
  { id: 'home', label: 'Home', icon: Home, keyHint: 'Alt+1' },
  { id: 'discover', label: 'Discover', icon: Compass, keyHint: 'Alt+2' },
  { id: 'installed', label: 'Installed', icon: PackageCheck, keyHint: 'Alt+3' },
  { id: 'updates', label: 'Updates', icon: RefreshCw, keyHint: 'Alt+4' },
  { id: 'repositories', label: 'Repositories', icon: Database, keyHint: 'Alt+5' },
  { id: 'settings', label: 'Settings', icon: Settings, keyHint: 'Alt+6' },
];

export const Sidebar: React.FC = () => {
  const { activeTab, setActiveTab } = useAppStore();
  const { data: dbStats, isSuccess: isDbReady } = useDatabaseStatus();
  const { data: sysInfo } = useSystemInfo();

  // Keyboard navigation shortcuts: Alt+1 to Alt+6
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key >= '1' && e.key <= '6') {
        const index = parseInt(e.key, 10) - 1;
        if (navItems[index]) {
          e.preventDefault();
          setActiveTab(navItems[index].id);
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setActiveTab]);

  return (
    <aside className="sidebar">
      {/* Brand Header */}
      <div className="brand-header">
        <div className="brand-icon">
          <span style={{ fontWeight: 800, fontSize: '15px' }}>T³</span>
        </div>
        <div className="brand-info">
          <h1 className="brand-title">T3 Hub</h1>
          <span className="brand-badge">Open Source • v0.1.0</span>
        </div>
      </div>

      {/* Navigation Links */}
      <nav className="nav-container">
        <div className="nav-section-title">MAIN NAVIGATION</div>
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`nav-button ${isActive ? 'nav-button-active' : ''}`}
              title={`${item.label} (${item.keyHint})`}
            >
              <div className="nav-button-left">
                <Icon size={18} className="nav-icon" />
                <span className="nav-label">{item.label}</span>
              </div>
              <span className="nav-keyhint">{item.keyHint}</span>
            </button>
          );
        })}
      </nav>

      {/* Footer System Status */}
      <div className="sidebar-footer">
        <div className="system-status-card">
          <div className="status-row">
            <span className="status-dot-active" />
            <span className="status-title">SQLite Engine Active</span>
          </div>
          <div className="status-subtext">
            <HardDrive size={13} style={{ display: 'inline', marginRight: 4 }} />
            {isDbReady && dbStats ? `v${dbStats.schema_version} • WAL Mode` : 'Connecting...'}
          </div>
          {sysInfo && (
            <div className="status-arch">
              {sysInfo.os.toUpperCase()} • {sysInfo.arch.toUpperCase()}
            </div>
          )}
        </div>
      </div>
    </aside>
  );
};
