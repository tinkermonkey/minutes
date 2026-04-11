import { useState } from 'react';
import { useSearch } from '../hooks/useSearch';
import { SpeakerFilterSelect } from '../components/search/SpeakerFilterSelect';
import { SearchResultList } from '../components/search/SearchResultList';
import { SessionDateFilter } from '../components/sessions/SessionDateFilter';
import type { SearchFilters } from '../types/search';

export function SearchRoute() {
  const [query, setQuery]     = useState('');
  const [filters, setFilters] = useState<SearchFilters>({
    speaker_id: null,
    start_date: null,
    end_date:   null,
  });
  const [hasSearched, setHasSearched] = useState(false);
  const search = useSearch();

  function handleSubmit() {
    if (!query.trim()) return;
    setHasSearched(true);
    search.mutate({ query, filters });
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') handleSubmit();
  }

  return (
    <div className="p-6 flex flex-col gap-4">
      <h1 className="text-2xl font-semibold text-gray-900">Search</h1>

      {/* Search form */}
      <div className="bg-white border border-gray-200 rounded-xl p-4 flex flex-col gap-3">
        {/* Query input row */}
        <div className="flex gap-2">
          <input
            type="text"
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search transcripts..."
            autoFocus
            disabled={search.isPending}
            className="flex-1 border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleSubmit}
            disabled={search.isPending || !query.trim()}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg disabled:opacity-50"
          >
            {search.isPending ? 'Searching...' : 'Search'}
          </button>
        </div>

        {/* Filters row */}
        <div className="flex items-center gap-3 flex-wrap">
          <SpeakerFilterSelect
            value={filters.speaker_id}
            onChange={id => setFilters(prev => ({ ...prev, speaker_id: id }))}
          />
          <SessionDateFilter
            startDate={filters.start_date}
            endDate={filters.end_date}
            onChange={(start, end) => setFilters(prev => ({ ...prev, start_date: start, end_date: end }))}
          />
        </div>
      </div>

      {/* Results */}
      <SearchResultList
        results={search.data ?? []}
        isPending={search.isPending}
        query={query}
        hasSearched={hasSearched}
        error={search.error ? String(search.error) : undefined}
      />
    </div>
  );
}
