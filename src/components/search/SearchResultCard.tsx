import { Link } from '@tanstack/react-router';
import { speakerColor } from '../../lib/speakerColor';
import { formatDate, formatTime } from '../../lib/format';
import { RelevanceBar } from './RelevanceBar';
import type { SearchResult } from '../../types/search';

interface Props {
  result: SearchResult;
}

export function SearchResultCard({ result }: Props) {
  const label =
    result.display_name
    ?? (result.speaker_id !== null ? `Speaker ${result.speaker_id}` : 'Unknown');
  const chipColor =
    result.speaker_id !== null
      ? speakerColor(result.speaker_id)
      : 'bg-gray-100 text-gray-600';

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4 mb-3 hover:shadow-sm transition-shadow">
      {/* Top row: speaker chip + relevance */}
      <div className="flex items-center justify-between mb-2">
        <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${chipColor}`}>
          {label}
        </span>
        <RelevanceBar score={result.score} />
      </div>

      {/* Transcript text */}
      <p className="text-sm text-gray-900 mb-3 leading-relaxed">
        {result.transcript_text}
      </p>

      {/* Session context */}
      <div className="flex items-center gap-2 text-xs text-gray-400">
        <span>{formatDate(result.session_created_at)} · {formatTime(result.session_created_at)}</span>
        <Link
          to="/sessions/$sessionId"
          params={{ sessionId: String(result.session_id) }}
          className="text-blue-500 hover:text-blue-700 underline"
        >
          View session
        </Link>
      </div>
    </div>
  );
}
