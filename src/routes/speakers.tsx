import { useState } from 'react';
import { useSpeakers, useDeleteSpeaker, useResetRegistry } from '../hooks/useSpeakers';
import { SpeakerRow } from '../components/speakers/SpeakerRow';
import { SimilarSpeakersPanel } from '../components/speakers/SimilarSpeakersPanel';
import { SpeakerDetailPanel } from '../components/speakers/SpeakerDetailPanel';
import { DeleteConfirmModal } from '../components/speakers/DeleteConfirmModal';
import { ResetRegistryModal } from '../components/speakers/ResetRegistryModal';
import { SpeakerCardSkeleton } from '../components/speakers/SpeakerCardSkeleton';
import { QueryError } from '../components/QueryError';
import type { Speaker } from '../types/speaker';

type FilterTab = 'all' | 'recognized' | 'unrecognized';
type RightPanel = 'similar' | 'detail' | null;

export function SpeakersRoute() {
  const { data: speakers = [], isLoading, isError, error, refetch } = useSpeakers();
  const deleteSpeaker = useDeleteSpeaker();
  const resetRegistry = useResetRegistry();

  const [search, setSearch]           = useState('');
  const [filter, setFilter]           = useState<FilterTab>('all');
  const [selectedId, setSelectedId]   = useState<number | null>(null);
  const [detailId, setDetailId]       = useState<number | null>(null);
  const [rightPanel, setRightPanel]   = useState<RightPanel>(null);
  const [deleteTarget, setDeleteTarget] = useState<Speaker | null>(null);
  const [deleteError, setDeleteError]  = useState<string | null>(null);
  const [showReset, setShowReset]      = useState(false);
  const [resetError, setResetError]    = useState<string | null>(null);

  // --- Speaker grouping (recognized first, unrecognized always at bottom) ---
  const filtered = speakers
    .filter(s =>
      filter === 'all' ||
      (filter === 'recognized' && s.display_name !== null) ||
      (filter === 'unrecognized' && s.display_name === null)
    )
    .filter(s =>
      !search || (s.display_name ?? '').toLowerCase().includes(search.toLowerCase())
    );

  const recognized   = filtered.filter(s => s.display_name !== null).sort((a, b) => b.last_seen_at - a.last_seen_at);
  const unrecognized = filtered.filter(s => s.display_name === null).sort((a, b) => b.last_seen_at - a.last_seen_at);

  const selectedSpeaker = selectedId !== null ? speakers.find(s => s.speech_swift_id === selectedId) ?? null : null;
  const detailSpeaker   = detailId   !== null ? speakers.find(s => s.speech_swift_id === detailId)   ?? null : null;

  // --- Panel interactions ---
  function handleSelectRow(id: number) {
    if (selectedId === id && rightPanel === 'similar') {
      setSelectedId(null); setRightPanel(null);
    } else {
      setSelectedId(id); setDetailId(null); setRightPanel('similar');
    }
  }

  function handleOpenDetail(id: number) {
    setDetailId(id); setSelectedId(null); setRightPanel('detail');
  }

  function handleFindSimilar() {
    if (detailId !== null) {
      setSelectedId(detailId); setDetailId(null); setRightPanel('similar');
    }
  }

  function handleClosePanel() {
    setSelectedId(null); setDetailId(null); setRightPanel(null);
  }

  // --- Loading / error states ---
  if (isLoading) return <div className="p-6"><SpeakerCardSkeleton /></div>;
  if (isError) return <QueryError message={error instanceof Error ? error.message : String(error)} onRetry={() => refetch()} />;

  const filterTabs: { key: FilterTab; label: string }[] = [
    { key: 'all', label: 'All' },
    { key: 'recognized', label: 'Recognized' },
    { key: 'unrecognized', label: 'Unrecognized' },
  ];

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Page header */}
      <div className="flex items-center gap-3 px-6 py-4 bg-white border-b border-gray-200 flex-shrink-0">
        <h1 className="text-xl font-semibold text-gray-900">Speakers</h1>
        <span className="text-sm text-gray-500 bg-gray-100 rounded-full px-3 py-0.5">
          {speakers.length} speakers
          {unrecognized.length > 0 && ` · ${unrecognized.length} unrecognized`}
        </span>
        <span className="flex-1" />
        <button
          onClick={() => { setResetError(null); setShowReset(true); }}
          className="px-3 py-1.5 text-xs font-medium text-red-600 border border-red-200 hover:bg-red-50 rounded-lg transition-colors"
        >
          Reset Registry
        </button>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-3 px-6 py-2.5 bg-white border-b border-gray-200 flex-shrink-0">
        <input
          type="text"
          placeholder="Search speakers…"
          value={search}
          onChange={e => setSearch(e.target.value)}
          className="w-52 px-3 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        />
        <div className="flex items-center bg-gray-100 rounded-lg p-0.5 gap-0.5">
          {filterTabs.map(tab => (
            <button
              key={tab.key}
              onClick={() => setFilter(tab.key)}
              className={`px-3 py-1 text-xs font-medium rounded-md transition-colors ${
                filter === tab.key
                  ? 'bg-white text-gray-900 shadow-sm'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
        <span className="text-xs text-gray-400">Sort: Last seen ↓</span>
        <span className="text-xs text-gray-300 hidden lg:block">↑ sort applies within each group</span>
      </div>

      {/* Body: list + optional right panel */}
      <div className="flex flex-1 overflow-hidden">
        {/* Speaker list */}
        <div className="flex-1 overflow-y-auto">
          {/* Table header */}
          <div className="flex items-center h-9 px-6 gap-3 bg-gray-50 border-b border-gray-200 text-xs font-semibold text-gray-500 uppercase tracking-wide sticky top-0 z-10">
            <div className="w-5" />
            <span className="min-w-[160px]">Speaker</span>
            <span className="ml-1">Sessions</span>
            <span className="flex-1" />
            <span className="w-28 text-right">Last seen</span>
            <span className="max-w-xs hidden lg:block ml-4">Recent transcript</span>
          </div>

          {/* Empty state */}
          {speakers.length === 0 && (
            <div className="flex flex-col items-center justify-center py-20 text-center px-6">
              <p className="text-gray-500 font-medium mb-1">No speakers yet</p>
              <p className="text-sm text-gray-400">Start a recording session to detect speakers.</p>
            </div>
          )}

          {/* Recognized rows */}
          {recognized.map(s => (
            <SpeakerRow
              key={s.id}
              speaker={s}
              isSelected={selectedId === s.speech_swift_id}
              onSelect={() => handleSelectRow(s.speech_swift_id)}
              onOpenDetail={() => handleOpenDetail(s.speech_swift_id)}
            />
          ))}

          {/* Unrecognized section header */}
          {unrecognized.length > 0 && (
            <div className="flex items-center gap-2 px-6 py-2 bg-amber-50 border-y border-amber-200 sticky top-9 z-10">
              <span className="text-amber-600 text-xs flex-shrink-0">⚠</span>
              <span className="text-xs font-semibold text-amber-800">
                Unrecognized ({unrecognized.length})
              </span>
              <span className="text-xs text-amber-600 hidden lg:block">
                — give them a name to train the model
              </span>
            </div>
          )}

          {/* Unrecognized rows */}
          {unrecognized.map(s => (
            <SpeakerRow
              key={s.id}
              speaker={s}
              isSelected={selectedId === s.speech_swift_id}
              onSelect={() => handleSelectRow(s.speech_swift_id)}
              onOpenDetail={() => handleOpenDetail(s.speech_swift_id)}
            />
          ))}
        </div>

        {/* Right panel */}
        {rightPanel === 'similar' && selectedSpeaker && (
          <SimilarSpeakersPanel
            selectedSpeaker={selectedSpeaker}
            onClose={handleClosePanel}
            onMergeSuccess={handleClosePanel}
            onDelete={s => { setDeleteError(null); setDeleteTarget(s); }}
          />
        )}
        {rightPanel === 'detail' && detailSpeaker && (
          <SpeakerDetailPanel
            speaker={detailSpeaker}
            onClose={handleClosePanel}
            onFindSimilar={handleFindSimilar}
            onDelete={s => { setDeleteError(null); setDeleteTarget(s); }}
          />
        )}
      </div>

      {/* Delete modal */}
      {deleteTarget && (
        <DeleteConfirmModal
          speaker={deleteTarget}
          isPending={deleteSpeaker.isPending}
          error={deleteError}
          onConfirm={async () => {
            setDeleteError(null);
            try {
              await deleteSpeaker.mutateAsync(deleteTarget.speech_swift_id);
              setDeleteTarget(null);
              handleClosePanel();
            } catch (e) {
              setDeleteError(e instanceof Error ? e.message : String(e));
            }
          }}
          onCancel={() => { setDeleteError(null); setDeleteTarget(null); }}
        />
      )}

      {/* Reset registry modal */}
      {showReset && (
        <ResetRegistryModal
          isPending={resetRegistry.isPending}
          error={resetError}
          onConfirm={async () => {
            setResetError(null);
            try {
              await resetRegistry.mutateAsync();
              setShowReset(false);
              handleClosePanel();
            } catch (e) {
              setResetError(e instanceof Error ? e.message : String(e));
            }
          }}
          onCancel={() => { setResetError(null); setShowReset(false); }}
        />
      )}
    </div>
  );
}
