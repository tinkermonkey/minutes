import { SearchResultCard } from './SearchResultCard';
import { SearchResultSkeleton } from './SearchResultSkeleton';
import type { SearchResult } from '../../types/search';

interface Props {
  results:     SearchResult[];
  isPending:   boolean;
  query:       string;
  hasSearched: boolean;
  error?:      string;
}

export function SearchResultList({ results, isPending, query, hasSearched, error }: Props) {
  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg px-4 py-3 text-red-800 text-sm">
        Search failed: {error}. Check that speech-swift is running.
      </div>
    );
  }

  if (isPending) {
    return <SearchResultSkeleton />;
  }

  if (!hasSearched) {
    return (
      <div className="flex items-center justify-center py-12 text-gray-400 text-sm">
        Search your transcript history
      </div>
    );
  }

  if (results.length === 0) {
    return (
      <p className="text-gray-400 text-sm text-center py-12">
        No results found for &ldquo;{query}&rdquo;. Try different keywords or broaden the date range.
      </p>
    );
  }

  return (
    <div>
      <p className="text-sm text-gray-400 mb-3">
        {results.length} {results.length === 1 ? 'result' : 'results'} for &ldquo;{query}&rdquo;
      </p>
      {results.map(r => (
        <SearchResultCard key={r.segment_id} result={r} />
      ))}
    </div>
  );
}
