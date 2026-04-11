import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { SearchResult, SearchFilters } from '../types/search';

export function useSearch() {
  return useMutation({
    mutationFn: ({ query, filters }: { query: string; filters: SearchFilters }): Promise<SearchResult[]> =>
      invoke('search_segments', { query, filters }),
  });
}
