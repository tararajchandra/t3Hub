import React, { useRef, useEffect } from 'react';
import { Search, Sun, Moon, Database, Shield } from 'lucide-react';
import { useAppStore } from '../../hooks/useAppStore';
import { useRepositories } from '../../hooks/useBackend';

export const Header: React.FC = () => {
  const { activeTab, searchQuery, setSearchQuery, theme, setTheme, setActiveTab } = useAppStore();
  const searchInputRef = useRef<HTMLInputElement>(null);
  const { data: repos } = useRepositories();

  // Keyboard shortcut: Ctrl+K or / to focus search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey && e.key === 'k') || (e.key === '/' && document.activeElement !== searchInputRef.current)) {
        e.preventDefault();
        setActiveTab('discover');
        setTimeout(() => searchInputRef.current?.focus(), 50);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setActiveTab]);

  const toggleTheme = () => {
    const nextTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(nextTheme);
  };

  const getPageTitle = () => {
    switch (activeTab) {
      case 'home':
        return 'Overview & Spotlight';
      case 'discover':
        return 'Discover Applications';
      case 'installed':
        return 'Installed Packages';
      case 'updates':
        return 'Software Updates';
      case 'repositories':
        return 'Package Repositories';
      case 'settings':
        return 'Platform Settings';
      default:
        return 'T3 Hub';
    }
  };

  const officialRepo = repos?.find((r) => r.repo_type === 'official');

  return (
    <header className="header">
      <div className="header-left">
        <h2 className="page-heading">{getPageTitle()}</h2>
      </div>

      <div className="header-center">
        <div className="search-box">
          <Search size={16} className="search-icon" />
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search open-source & commercial apps... (Ctrl+K)"
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              if (activeTab !== 'discover') {
                setActiveTab('discover');
              }
            }}
            className="search-input"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="search-clear-btn"
              title="Clear search"
            >
              ×
            </button>
          )}
        </div>
      </div>

      <div className="header-right">
        {/* Repository status badge */}
        <div className="repo-pill" title="Repository network status">
          <Database size={13} className="text-accent" />
          <span>{officialRepo?.enabled ? 'Official Repo Online' : 'Local Repo Only'}</span>
          <Shield size={12} className="text-success" style={{ marginLeft: 2 }} />
        </div>

        {/* Theme Toggle Button */}
        <button
          onClick={toggleTheme}
          className="theme-toggle-btn"
          title={`Switch to ${theme === 'dark' ? 'Light' : 'Dark'} mode`}
          aria-label="Toggle color theme"
        >
          {theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}
        </button>
      </div>
    </header>
  );
};
