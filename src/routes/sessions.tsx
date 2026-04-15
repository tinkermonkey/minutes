import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useSessions, useDeleteAllSessions } from '../hooks/useSessions';
import { ParticipantChips } from '../components/sessions/ParticipantChips';
import { SourceBadge } from '../components/sessions/SourceBadge';
import { SortableHeader } from '../components/sessions/SortableHeader';
import { SessionDateFilter } from '../components/sessions/SessionDateFilter';
import { SessionTableSkeleton } from '../components/sessions/SessionTableSkeleton';
import { DeleteAllSessionsModal } from '../components/sessions/DeleteAllSessionsModal';
import { QueryError } from '../components/QueryError';
import { formatDate, formatTime, formatDuration } from '../lib/format';
import type { SessionFilter, SortBy } from '../types/session';

export function SessionsRoute() {
  const navigate = useNavigate();
  const [filter, setFilter] = useState<SessionFilter>({
    start_date: null,
    end_date:   null,
    sort_by:    'date',
    sort_dir:   'desc',
    page:       1,
    page_size:  20,
  });
  const [showDeleteAllModal, setShowDeleteAllModal] = useState(false);
  const [deleteAllError, setDeleteAllError] = useState<string | null>(null);

  const { data, isLoading, isFetching, isError, error, refetch } = useSessions(filter);
  const deleteAllSessions = useDeleteAllSessions();
  const sessions = data?.sessions ?? [];
  const totalCount = data?.total_count ?? 0;
  const totalPages = Math.ceil(totalCount / filter.page_size);

  function updateFilter(patch: Partial<SessionFilter>) {
    setFilter(prev => ({ ...prev, ...patch, page: 1 }));
  }

  function handleSort(field: SortBy) {
    setFilter(prev => ({
      ...prev,
      sort_by:  field,
      sort_dir: prev.sort_by === field && prev.sort_dir === 'desc' ? 'asc' : 'desc',
      page:     1,
    }));
  }

  const startItem = (filter.page - 1) * filter.page_size + 1;
  const endItem   = Math.min(filter.page * filter.page_size, totalCount);

  return (
    <div className="p-6 flex flex-col gap-4">
      <div className="flex items-center justify-between flex-wrap gap-3">
        <h1 className="text-2xl font-semibold text-gray-900">Session History</h1>
        <div className="flex items-center gap-3">
          <SessionDateFilter
            startDate={filter.start_date}
            endDate={filter.end_date}
            onChange={(start, end) => updateFilter({ start_date: start, end_date: end })}
          />
          {totalCount > 0 && (
            <button
              onClick={() => { setDeleteAllError(null); setShowDeleteAllModal(true); }}
              className="px-3 py-1.5 text-xs font-medium text-red-600 border border-red-200 hover:bg-red-50 rounded-lg transition-colors"
            >
              Delete All
            </button>
          )}
        </div>
      </div>

      {isLoading ? (
        <SessionTableSkeleton />
      ) : isError ? (
        <QueryError
          message={error instanceof Error ? error.message : String(error)}
          onRetry={() => refetch()}
        />
      ) : (
        <div className={isFetching ? 'opacity-60 pointer-events-none' : ''}>
          <table className="w-full text-sm border-collapse">
            <thead>
              <tr className="border-b border-gray-200">
                <th className="text-left p-3">
                  <SortableHeader label="Date" field="date" sortBy={filter.sort_by} sortDir={filter.sort_dir} onSort={handleSort} />
                </th>
                <th className="text-left p-3">
                  <SortableHeader label="Duration" field="duration" sortBy={filter.sort_by} sortDir={filter.sort_dir} onSort={handleSort} />
                </th>
                <th className="text-left p-3 font-medium text-gray-700">Participants</th>
                <th className="text-left p-3 font-medium text-gray-700">Source</th>
              </tr>
            </thead>
            <tbody>
              {sessions.map((session, i) => (
                <tr
                  key={session.id}
                  className={`cursor-pointer hover:bg-gray-50 border-b border-gray-100 ${i % 2 === 1 ? 'bg-gray-50/50' : 'bg-white'}`}
                  onClick={() => navigate({ to: '/sessions/$sessionId', params: { sessionId: String(session.id) } })}
                >
                  <td className="p-3">
                    <div className="font-medium text-gray-900">{formatDate(session.created_at)}</div>
                    <div className="text-xs text-gray-500">{formatTime(session.created_at)}</div>
                  </td>
                  <td className="p-3 text-gray-700">{formatDuration(session.duration_ms)}</td>
                  <td className="p-3"><ParticipantChips participants={session.participants} /></td>
                  <td className="p-3"><SourceBadge source={session.source} /></td>
                </tr>
              ))}
            </tbody>
          </table>

          {sessions.length === 0 && (
            <p className="text-gray-400 text-center py-12 text-sm">
              {filter.start_date || filter.end_date
                ? 'No sessions found in this date range. Try adjusting the filter.'
                : 'No sessions yet. Start a recording to create your first session.'}
            </p>
          )}

          {totalCount > filter.page_size && (
            <div className="flex items-center justify-between mt-4 text-sm text-gray-600">
              <span>Showing {startItem}–{endItem} of {totalCount} sessions</span>
              <div className="flex gap-2">
                <button
                  onClick={() => setFilter(prev => ({ ...prev, page: prev.page - 1 }))}
                  disabled={filter.page <= 1}
                  className="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-40"
                >
                  Previous
                </button>
                <button
                  onClick={() => setFilter(prev => ({ ...prev, page: prev.page + 1 }))}
                  disabled={filter.page >= totalPages}
                  className="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-40"
                >
                  Next
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {showDeleteAllModal && (
        <DeleteAllSessionsModal
          isPending={deleteAllSessions.isPending}
          error={deleteAllError}
          onConfirm={async () => {
            setDeleteAllError(null);
            try {
              await deleteAllSessions.mutateAsync();
              setShowDeleteAllModal(false);
            } catch (e) {
              setDeleteAllError(e instanceof Error ? e.message : String(e));
            }
          }}
          onCancel={() => { setDeleteAllError(null); setShowDeleteAllModal(false); }}
        />
      )}
    </div>
  );
}
