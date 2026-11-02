import { create } from 'zustand';
import type { NavigationTab, ThemeMode } from '../types';

interface AppStore {
  activeTab: NavigationTab;
  setActiveTab: (tab: NavigationTab) => void;
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  selectedCategory: string;
  setSelectedCategory: (cat: string) => void;
  selectedPackageId: string | null;
  setSelectedPackageId: (id: string | null) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  activeTab: 'home',
  setActiveTab: (tab) => set({ activeTab: tab }),
  theme: (localStorage.getItem('t3hub_theme') as ThemeMode) || 'dark',
  setTheme: (theme) => {
    localStorage.setItem('t3hub_theme', theme);
    set({ theme });
  },
  searchQuery: '',
  setSearchQuery: (query) => set({ searchQuery: query }),
  selectedCategory: 'All',
  setSelectedCategory: (cat) => set({ selectedCategory: cat }),
  selectedPackageId: null,
  setSelectedPackageId: (id) => set({ selectedPackageId: id }),
}));
