import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { tauriService } from '../services/tauri';

export function useSystemInfo() {
  return useQuery({
    queryKey: ['systemInfo'],
    queryFn: () => tauriService.getSystemInfo(),
    staleTime: Infinity,
  });
}

export function useDatabaseStatus() {
  return useQuery({
    queryKey: ['databaseStatus'],
    queryFn: () => tauriService.getDatabaseStatus(),
    refetchInterval: 10000,
  });
}

export function useSettings() {
  return useQuery({
    queryKey: ['settings'],
    queryFn: () => tauriService.getSettings(),
  });
}

export function useRepositories() {
  return useQuery({
    queryKey: ['repositories'],
    queryFn: () => tauriService.getRepositories(),
  });
}

export function useInstalledApps() {
  return useQuery({
    queryKey: ['installedApps'],
    queryFn: () => tauriService.getInstalledApplications(),
  });
}

export function useOperationsLog() {
  return useQuery({
    queryKey: ['operationsLog'],
    queryFn: () => tauriService.getOperationsLog(),
    refetchInterval: 5000,
  });
}

export function useDetectedApps() {
  return useQuery({
    queryKey: ['detectedApps'],
    queryFn: () => tauriService.getDetectedInstalledApps(),
    staleTime: 60000,
  });
}

export function useUninstallPackage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (packageId: string) => tauriService.uninstallPackage(packageId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['installedApps'] });
      queryClient.invalidateQueries({ queryKey: ['catalogPackages'] });
      queryClient.invalidateQueries({ queryKey: ['databaseStatus'] });
      queryClient.invalidateQueries({ queryKey: ['operationsLog'] });
    },
  });
}

export function useAvailableUpdates() {
  return useQuery({
    queryKey: ['availableUpdates'],
    queryFn: () => tauriService.checkUpdates(),
    refetchInterval: 30000,
  });
}

export function useToggleRepository() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      tauriService.toggleRepository(id, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['catalogPackages'] });
      queryClient.invalidateQueries({ queryKey: ['availableUpdates'] });
    },
  });
}

export function useRemoveRepository() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => tauriService.removeRepository(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['catalogPackages'] });
      queryClient.invalidateQueries({ queryKey: ['databaseStatus'] });
      queryClient.invalidateQueries({ queryKey: ['availableUpdates'] });
    },
  });
}

export function useUpdateRepositoryTrust() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, trustPolicy }: { id: string; trustPolicy: string }) =>
      tauriService.updateRepositoryTrust(id, trustPolicy),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });
}

export function useUpdateSetting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ key, value }: { key: string; value: string }) =>
      tauriService.updateSetting(key, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      queryClient.invalidateQueries({ queryKey: ['databaseStatus'] });
    },
  });
}

export function useCatalogSearch(query?: string, category?: string, repositoryId?: string) {
  return useQuery({
    queryKey: ['catalogSearch', query, category, repositoryId],
    queryFn: () => tauriService.searchCatalog(query, category, repositoryId),
  });
}

export function usePackageDetails(packageId: string | null) {
  return useQuery({
    queryKey: ['packageDetails', packageId],
    queryFn: () => (packageId ? tauriService.getPackageDetails(packageId) : Promise.resolve(null)),
    enabled: !!packageId,
  });
}

export function useSyncRepositories() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => tauriService.syncRepositories(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['catalogSearch'] });
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['databaseStatus'] });
    },
  });
}

export function useAddRepository() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name, url, repoType }: { id: string; name: string; url: string; repoType?: string }) =>
      tauriService.addRepository(id, name, url, repoType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['databaseStatus'] });
    },
  });
}

export function useInstallPackage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ packageId, version }: { packageId: string; version?: string }) =>
      tauriService.installPackage(packageId, version),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['installedApps'] });
      queryClient.invalidateQueries({ queryKey: ['catalogSearch'] });
      queryClient.invalidateQueries({ queryKey: ['operationsLog'] });
      queryClient.invalidateQueries({ queryKey: ['databaseStatus'] });
    },
  });
}
