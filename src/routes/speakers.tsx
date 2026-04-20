import { useState } from 'react';
import { useSpeakers, useDeleteSpeaker, useMergeSpeakers, useResetRegistry } from '../hooks/useSpeakers';
import { SpeakerRow } from '../components/speakers/SpeakerRow';
import { SimilarSpeakersPanel } from '../components/speakers/SimilarSpeakersPanel';
import { SpeakerDetailPanel } from '../components/speakers/SpeakerDetailPanel';
import { DeleteConfirmModal } from '../components/speakers/DeleteConfirmModal';
import { BulkDeleteModal } from '../components/speakers/BulkDeleteModal';
import { ResetRegistryModal } from '../components/speakers/ResetRegistryModal';
import { SpeakerCardSkeleton } from '../components/speakers/SpeakerCardSkeleton';
import { QueryError } from '../components/QueryError';
import type { Speaker } from '../types/speaker';

type FilterTab = 'all' | 'recognized' | 'unrecognized';
type RightPanel = 'similar' | 'detail' | null;
type BulkMergeState = 'idle' | 'confirm' | 'running';

function Spinner() {
  return <div className="w-3 h-3 border border-blue-300 border-t-blue-600 rounded-full animate-spin" />;
}

function ChevronDown() {
  return (
    <svg className="w-3 h-3 text-gray-500" viewBox="0 0 12 12" fill="none" aria-hidden>
      <path d="M2 4l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function SpeakersRoute() {
  const { data: speakers = [], isLoading, isError, error, refetch } = useSpeakers();
  const deleteSpeaker  = useDeleteSpeaker();
  const mergeSpeakers  = useMergeSpeakers();
  const resetRegistry  = useResetRegistry();

  // --- Single-speaker panel state ---
  const [search, setSearch]           = useState('');
  const [filter, setFilter]           = useState<FilterTab>('all');
  const [selectedId, setSelectedId]   = useState<number | null>(null);
  const [detailId, setDetailId]       = useState<number | null>(null);
  const [rightPanel, setRightPanel]   = useState<RightPanel>(null);
  const [deleteTarget, setDeleteTarget]           = useState<Speaker | null>(null);
  const [deleteTargetIsSelected, setDeleteTargetIsSelected] = useState(false);
  const [deleteError, setDeleteError]             = useState<string | null>(null);
  const [showReset, setShowReset]     = useState(false);
  const [resetError, setResetError]   = useState<string | null>(null);

  // --- Bulk selection state ---
  const [checkedIds, setCheckedIds]               = useState<Set<number>>(new Set());
  const [bulkActionsOpen, setBulkActionsOpen]     = useState(false);
  const [bulkMergeState, setBulkMergeState]       = useState<BulkMergeState>('idle');
  const [bulkMergeError, setBulkMergeError]       = useState<string | null>(null);
  const [showBulkDelete, setShowBulkDelete]       = useState(false);
  const [bulkDeleteProgress, setBulkDeleteProgress] = useState<{ done: number; total: number } | null>(null);
  const [bulkDeleteError, setBulkDeleteError]     = useState<string | null>(null);

  // --- Speaker grouping ---
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
  const allVisible   = [...recognized, ...unrecognized];

  const selectedSpeaker = selectedId !== null ? speakers.find(s => s.speech_swift_id === selectedId) ?? null : null;
  const detailSpeaker   = detailId   !== null ? speakers.find(s => s.speech_swift_id === detailId)   ?? null : null;

  // --- Single-panel interactions ---
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

  // --- Bulk selection helpers ---
  function toggleCheck(id: number) {
    setCheckedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }

  function clearChecked() {
    setCheckedIds(new Set());
  }

  function resetBulkState() {
    setCheckedIds(new Set());
    setBulkMergeState('idle');
    setBulkMergeError(null);
  }

  const allVisibleChecked = allVisible.length > 0 && allVisible.every(s => checkedIds.has(s.speech_swift_id));
  const someVisibleChecked = allVisible.some(s => checkedIds.has(s.speech_swift_id));

  function toggleSelectAll() {
    if (allVisibleChecked) {
      setCheckedIds(new Set());
    } else {
      setCheckedIds(new Set(allVisible.map(s => s.speech_swift_id)));
    }
  }

  // --- Bulk merge ---
  function handleBulkMergeClick() {
    const checkedSpeakers = speakers.filter(s => checkedIds.has(s.speech_swift_id));
    if (checkedSpeakers.length < 2) {
      setBulkMergeError('Select at least 2 speakers to merge.');
      return;
    }
    const named = checkedSpeakers.filter(s => s.display_name !== null);
    if (named.length === 0) {
      setBulkMergeError('At least one recognized speaker must be selected as the merge target.');
      return;
    }
    if (named.length > 1) {
      setBulkMergeError('Select only one recognized speaker as the merge target; the others will be merged into it.');
      return;
    }
    setBulkMergeError(null);
    setBulkMergeState('confirm');
  }

  async function handleConfirmBulkMerge() {
    const checkedSpeakers = speakers.filter(s => checkedIds.has(s.speech_swift_id));
    const winner = checkedSpeakers.find(s => s.display_name !== null)!;
    const toMerge = checkedSpeakers.filter(s => s.speech_swift_id !== winner.speech_swift_id);

    setBulkMergeState('running');
    setBulkMergeError(null);
    try {
      for (const src of toMerge) {
        await mergeSpeakers.mutateAsync({ srcId: src.speech_swift_id, dstId: winner.speech_swift_id });
      }
      resetBulkState();
    } catch (e) {
      setBulkMergeError(e instanceof Error ? e.message : String(e));
      setBulkMergeState('confirm');
    }
  }

  // --- Bulk delete ---
  async function handleConfirmBulkDelete() {
    const checkedSpeakers = speakers.filter(s => checkedIds.has(s.speech_swift_id));
    const total = checkedSpeakers.length;
    setBulkDeleteProgress({ done: 0, total });
    setBulkDeleteError(null);

    for (let i = 0; i < checkedSpeakers.length; i++) {
      try {
        await deleteSpeaker.mutateAsync(checkedSpeakers[i].speech_swift_id);
        setBulkDeleteProgress({ done: i + 1, total });
      } catch (e) {
        setBulkDeleteError(e instanceof Error ? e.message : String(e));
        return;
      }
    }

    resetBulkState();
    setShowBulkDelete(false);
    setBulkDeleteProgress(null);
    handleClosePanel();
  }

  // --- Loading / error states ---
  if (isLoading) return <div className="p-6"><SpeakerCardSkeleton /></div>;
  if (isError) return <QueryError message={error instanceof Error ? error.message : String(error)} onRetry={() => refetch()} />;

  const filterTabs: { key: FilterTab; label: string }[] = [
    { key: 'all', label: 'All' },
    { key: 'recognized', label: 'Recognized' },
    { key: 'unrecognized', label: 'Unrecognized' },
  ];

  // Derive merge confirmation details once (used in JSX)
  const checkedSpeakers    = speakers.filter(s => checkedIds.has(s.speech_swift_id));
  const bulkMergeWinner    = checkedSpeakers.find(s => s.display_name !== null) ?? null;
  const bulkMergeAbsorbing = bulkMergeWinner
    ? checkedSpeakers.filter(s => s.speech_swift_id !== bulkMergeWinner.speech_swift_id)
    : [];

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

      {/* Bulk action bar — visible when any speakers are checked */}
      {checkedIds.size > 0 && (
        <div className="flex-shrink-0 bg-blue-50 border-b border-blue-200">
          <div className="flex items-center gap-3 px-6 h-11">
            <span className="text-sm font-medium text-blue-900">
              {checkedIds.size} selected
            </span>
            <button
              onClick={clearChecked}
              className="text-xs text-blue-600 hover:text-blue-800 underline"
            >
              Clear
            </button>
            <span className="flex-1" />

            {bulkMergeState === 'running' ? (
              <span className="text-xs text-blue-700 flex items-center gap-1.5">
                <Spinner /> Merging…
              </span>
            ) : (
              <div className="relative">
                <button
                  onClick={() => setBulkActionsOpen(o => !o)}
                  disabled={bulkMergeState !== 'idle'}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 transition-colors"
                >
                  Actions <ChevronDown />
                </button>
                {bulkActionsOpen && (
                  <>
                    <div className="fixed inset-0 z-10" onClick={() => setBulkActionsOpen(false)} />
                    <div className="absolute right-0 mt-1 w-44 bg-white border border-gray-200 rounded-lg shadow-lg z-20 overflow-hidden">
                      <button
                        onClick={() => { setBulkActionsOpen(false); handleBulkMergeClick(); }}
                        className="w-full px-4 py-2.5 text-sm text-left text-gray-700 hover:bg-gray-50"
                      >
                        Merge Selected
                      </button>
                      <button
                        onClick={() => { setBulkActionsOpen(false); setBulkDeleteError(null); setBulkDeleteProgress(null); setShowBulkDelete(true); }}
                        className="w-full px-4 py-2.5 text-sm text-left text-red-600 hover:bg-red-50"
                      >
                        Delete Selected
                      </button>
                    </div>
                  </>
                )}
              </div>
            )}
          </div>

          {/* Merge validation error */}
          {bulkMergeError && bulkMergeState === 'idle' && (
            <div className="px-6 py-2 border-t border-amber-200 bg-amber-50 flex items-center gap-2">
              <span className="text-xs text-amber-800 flex-1">{bulkMergeError}</span>
              <button
                onClick={() => setBulkMergeError(null)}
                className="text-amber-600 hover:text-amber-800 text-sm leading-none"
              >
                ×
              </button>
            </div>
          )}

          {/* Merge confirmation */}
          {bulkMergeState === 'confirm' && bulkMergeWinner && (
            <div className="px-6 py-3 border-t border-amber-200 bg-amber-50">
              <p className="text-sm font-semibold text-amber-900 mb-0.5">Confirm Merge?</p>
              <p className="text-xs text-amber-800 mb-2">
                Merge{' '}
                <strong>{bulkMergeAbsorbing.map(s => s.display_name ?? `Unknown #${s.speech_swift_id}`).join(', ')}</strong>
                {' '}→{' '}
                <strong>{bulkMergeWinner.display_name}</strong>. All segments will be reassigned. This cannot be undone.
              </p>
              {bulkMergeError && (
                <p className="text-xs text-red-600 bg-red-50 border border-red-200 rounded p-1.5 mb-2">{bulkMergeError}</p>
              )}
              <div className="flex gap-2">
                <button
                  onClick={handleConfirmBulkMerge}
                  disabled={mergeSpeakers.isPending}
                  className="px-3 py-1.5 text-xs font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors disabled:opacity-50"
                >
                  {mergeSpeakers.isPending ? 'Merging…' : 'Confirm Merge'}
                </button>
                <button
                  onClick={() => { setBulkMergeState('idle'); setBulkMergeError(null); }}
                  className="px-3 py-1.5 text-xs font-medium border border-gray-300 text-gray-700 hover:bg-white rounded-md transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Body: list + optional right panel */}
      <div className="flex flex-1 overflow-hidden">
        {/* Speaker list */}
        <div className="flex-1 overflow-y-auto">
          {/* Table header */}
          <div className="flex items-center h-9 px-6 gap-3 bg-gray-50 border-b border-gray-200 text-xs font-semibold text-gray-500 uppercase tracking-wide sticky top-0 z-10">
            <div
              className="w-5 flex-shrink-0 flex items-center justify-center cursor-pointer"
              onClick={toggleSelectAll}
            >
              <input
                type="checkbox"
                checked={allVisibleChecked}
                ref={el => { if (el) el.indeterminate = someVisibleChecked && !allVisibleChecked; }}
                onChange={toggleSelectAll}
                className="w-3.5 h-3.5 rounded border-gray-300 text-blue-600 cursor-pointer pointer-events-none"
              />
            </div>
            <div className="w-2.5 flex-shrink-0" />
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
              isChecked={checkedIds.has(s.speech_swift_id)}
              onSelect={() => handleSelectRow(s.speech_swift_id)}
              onCheck={() => toggleCheck(s.speech_swift_id)}
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
              isChecked={checkedIds.has(s.speech_swift_id)}
              onSelect={() => handleSelectRow(s.speech_swift_id)}
              onCheck={() => toggleCheck(s.speech_swift_id)}
              onOpenDetail={() => handleOpenDetail(s.speech_swift_id)}
            />
          ))}
        </div>

        {/* Right panel */}
        {rightPanel === 'similar' && selectedSpeaker && (
          <SimilarSpeakersPanel
            selectedSpeaker={selectedSpeaker}
            onClose={handleClosePanel}
            onDelete={s => { setDeleteError(null); setDeleteTargetIsSelected(s.speech_swift_id === selectedId); setDeleteTarget(s); }}
          />
        )}
        {rightPanel === 'detail' && detailSpeaker && (
          <SpeakerDetailPanel
            speaker={detailSpeaker}
            onClose={handleClosePanel}
            onFindSimilar={handleFindSimilar}
            onDelete={s => { setDeleteError(null); setDeleteTargetIsSelected(s.speech_swift_id === detailId); setDeleteTarget(s); }}
          />
        )}
      </div>

      {/* Single-speaker delete modal */}
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
              if (deleteTargetIsSelected) handleClosePanel();
            } catch (e) {
              setDeleteError(e instanceof Error ? e.message : String(e));
            }
          }}
          onCancel={() => { setDeleteError(null); setDeleteTarget(null); setDeleteTargetIsSelected(false); }}
        />
      )}

      {/* Bulk delete modal */}
      {showBulkDelete && (
        <BulkDeleteModal
          speakers={checkedSpeakers}
          isPending={deleteSpeaker.isPending}
          progress={bulkDeleteProgress}
          error={bulkDeleteError}
          onConfirm={handleConfirmBulkDelete}
          onCancel={() => {
            if (!deleteSpeaker.isPending) {
              setShowBulkDelete(false);
              setBulkDeleteProgress(null);
              setBulkDeleteError(null);
              resetBulkState();
            }
          }}
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
