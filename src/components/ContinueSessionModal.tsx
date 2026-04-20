import { useState, useMemo } from 'react';
import { useSessions } from '../hooks/useSessions';
import type { Session } from '../types/session';

interface Props {
  onClose: () => void;
  onResume: (sessionId: number) => void;
}

function formatDate(epochMs: number): string {
  return new Date(epochMs).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatDuration(durationMs: number | null): string {
  if (!durationMs) return '—';
  const totalSecs = Math.floor(durationMs / 1000);
  const mins = Math.floor(totalSecs / 60);
  const secs = totalSecs % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function SessionRow({
  session,
  isSelected,
  onSelect,
}: {
  session: Session;
  isSelected: boolean;
  onSelect: () => void;
}) {
  const label = session.label ?? formatDate(session.created_at);
  const participantCount = session.participants.length;

  return (
    <button
      type="button"
      onClick={onSelect}
      className={`w-full text-left px-4 py-3 border-b border-gray-100 last:border-b-0 transition-colors ${
        isSelected
          ? 'bg-blue-50 border-l-2 border-l-blue-500'
          : 'hover:bg-gray-50 border-l-2 border-l-transparent'
      }`}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className={`text-sm font-medium truncate ${isSelected ? 'text-blue-700' : 'text-gray-800'}`}>
            {label}
          </div>
          <div className="text-xs text-gray-400 mt-0.5">
            {formatDate(session.created_at)}
          </div>
        </div>
        <div className="flex items-center gap-3 flex-shrink-0 text-xs text-gray-500">
          <span title="Duration">{formatDuration(session.duration_ms)}</span>
          {participantCount > 0 && (
            <span title="Speakers" className="flex items-center gap-0.5">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="11"
                height="11"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="opacity-60"
              >
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
                <circle cx="9" cy="7" r="4" />
                <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                <path d="M16 3.13a4 4 0 0 1 0 7.75" />
              </svg>
              {participantCount}
            </span>
          )}
        </div>
      </div>
    </button>
  );
}

export function ContinueSessionModal({ onClose, onResume }: Props) {
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const { data, isLoading, isError } = useSessions({
    start_date: null,
    end_date: null,
    sort_by: 'date',
    sort_dir: 'desc',
    page: 0,
    page_size: 20,
  });

  const sessions: Session[] = useMemo(
    () => data?.sessions ?? [],
    [data],
  );

  function handleConfirm() {
    if (selectedId == null) return;
    onResume(selectedId);
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md flex flex-col max-h-[80vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-gray-100">
          <h2 className="text-base font-semibold text-gray-900">Continue a Session</h2>
          <button
            type="button"
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-lg leading-none p-1 rounded"
            aria-label="Close"
          >
            &#x2715;
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto">
          {isLoading && (
            <div className="px-5 py-8 text-center text-sm text-gray-400">Loading sessions...</div>
          )}
          {isError && (
            <div className="px-5 py-8 text-center text-sm text-red-500">Failed to load sessions.</div>
          )}
          {!isLoading && !isError && sessions.length === 0 && (
            <div className="px-5 py-8 text-center text-sm text-gray-400">No sessions yet.</div>
          )}
          {!isLoading && !isError && sessions.length > 0 && (
            <div>
              {sessions.map(session => (
                <SessionRow
                  key={session.id}
                  session={session}
                  isSelected={selectedId === session.id}
                  onSelect={() => setSelectedId(session.id)}
                />
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-gray-100">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            disabled={selectedId == null}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg transition-colors"
          >
            Resume Selected
          </button>
        </div>
      </div>
    </div>
  );
}
