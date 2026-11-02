import React, { useEffect } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Sidebar } from './components/layout/Sidebar';
import { Header } from './components/layout/Header';
import { HomePage } from './pages/Home/HomePage';
import { DiscoverPage } from './pages/Discover/DiscoverPage';
import { InstalledPage } from './pages/Installed/InstalledPage';
import { UpdatesPage } from './pages/Updates/UpdatesPage';
import { RepositoriesPage } from './pages/Repositories/RepositoriesPage';
import { SettingsPage } from './pages/Settings/SettingsPage';
import { useAppStore } from './hooks/useAppStore';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

const AppContent: React.FC = () => {
  const { activeTab, theme } = useAppStore();

  // Synchronize color theme with root document element
  useEffect(() => {
    const root = document.documentElement;
    const applyTheme = () => {
      root.classList.remove('theme-dark', 'theme-light');
      if (theme === 'system') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        root.classList.add(prefersDark ? 'theme-dark' : 'theme-light');
      } else {
        root.classList.add(`theme-${theme}`);
      }
    };

    applyTheme();

    if (theme === 'system') {
      const mql = window.matchMedia('(prefers-color-scheme: dark)');
      mql.addEventListener('change', applyTheme);
      return () => mql.removeEventListener('change', applyTheme);
    }
  }, [theme]);

  const renderActiveScreen = () => {
    switch (activeTab) {
      case 'home':
        return <HomePage />;
      case 'discover':
        return <DiscoverPage />;
      case 'installed':
        return <InstalledPage />;
      case 'updates':
        return <UpdatesPage />;
      case 'repositories':
        return <RepositoriesPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <HomePage />;
    }
  };

  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-viewport">
        <Header />
        <main className="content-area">{renderActiveScreen()}</main>
      </div>
    </div>
  );
};

export const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  );
};

export default App;
