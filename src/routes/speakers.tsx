import { useState } from 'react';
import { useSpeakers, useMergeSpeakers, useDeleteSpeaker, useResetRegistry } from '../hooks/useSpeakers';
import { SpeakerCard } from '../components/speakers/SpeakerCard';
import { MergeConfirmModal } from '../components/speakers/MergeConfirmModal';
import { DeleteConfirmModal } from '../components/speakers/DeleteConfirmModal';
import { ResetRegistryModal } from '../components/speakers/ResetRegistryModal';
import { SpeakerCardSkeleton } from '../components/speakers/SpeakerCardSkeleton';
import { QueryError } from '../components/QueryError';
import type { Speaker } from '../types/speaker';
import type { MergeState } from '../components/speakers/MergeSelectButton';

export function SpeakersRoute() {
  const { data: speakers = [], isLoading, isError, error, refetch } = useSpeakers();
  const [mergeState, setMergeState] = useState<MergeState>({ phase: 'idle' });
  const [deleteTarget, setDeleteTarget] = useState<Speaker | null>(null);
  const [mergeError, setMergeError] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [showResetModal, setShowResetModal] = useState(false);
  const [resetError, setResetError] = useState<string | null>(null);
  const mergeSpeakers = useMergeSpeakers();
  const deleteSpeaker = useDeleteSpeaker();
  const resetRegistry = useResetRegistry();

  const isRecording = false;

  function handleMergeSelect(selectedId: number) {
    setMergeError(null);
    if (mergeState.phase === 'idle') {
      setMergeState({ phase: 'selecting', srcId: selectedId });
    } else if (mergeState.phase === 'selecting') {
      if (selectedId === mergeState.srcId) {
        setMergeState({ phase: 'idle' });
      } else {
        setMergeState({ phase: 'confirming', srcId: mergeState.srcId, dstId: selectedId });
      }
    }
  }

  function getSrcSpeakerName(): string | null {
    if (mergeState.phase !== 'selecting' && mergeState.phase !== 'confirming') return null;
    const src = speakers.find(s => s.speech_swift_id === mergeState.srcId);
    if (!src) return null;
    return src.display_name ?? `Speaker ${src.speech_swift_id}`;
  }

  if (isLoading) {
    return (
      <div className="p-6">
        <SpeakerCardSkeleton />
      </div>
    );
  }

  if (isError) {
    return (
      <QueryError
        message={error instanceof Error ? error.message : String(error)}
        onRetry={() => refetch()}
      />
    );
  }

  return (
    <div className="p-6 flex flex-col gap-4">
      {/* Header */}
      <div className="flex items-center gap-3">
        <h1 className="text-2xl font-semibold text-gray-900">Speaker Registry</h1>
        <span className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded-full">
          {speakers.length} {speakers.length === 1 ? 'speaker' : 'speakers'}
        </span>
        <div className="ml-auto">
          <button
            onClick={() => { setResetError(null); setShowResetModal(true); }}
            className="px-3 py-1.5 text-xs font-medium text-red-600 border border-red-200 hover:bg-red-50 rounded-lg transition-colors"
          >
            Reset Registry
          </button>
        </div>
      </div>

      {/* Empty state */}
      {speakers.length === 0 && (
        <p className="text-gray-500">
          No speakers yet. Start a recording session to detect speakers.
        </p>
      )}

      {/* Speaker list */}
      <div className="flex flex-col gap-3">
        {speakers.map(speaker => (
          <SpeakerCard
            key={speaker.id}
            speaker={speaker}
            mergeState={mergeState}
            onMergeSelect={handleMergeSelect}
            onMergeCancel={() => setMergeState({ phase: 'idle' })}
            onDeleteClick={() => setDeleteTarget(speaker)}
            srcSpeakerName={getSrcSpeakerName()}
            isRecording={isRecording}
          />
        ))}
      </div>

      {/* Merge confirm modal */}
      {mergeState.phase === 'confirming' && (() => {
        const src = speakers.find(s => s.speech_swift_id === mergeState.srcId);
        const dst = speakers.find(s => s.speech_swift_id === mergeState.dstId);
        if (!src || !dst) return null;
        return (
          <MergeConfirmModal
            src={src}
            dst={dst}
            isPending={mergeSpeakers.isPending}
            error={mergeError}
            onConfirm={async () => {
              setMergeError(null);
              try {
                await mergeSpeakers.mutateAsync({
                  srcId: mergeState.srcId,
                  dstId: mergeState.dstId,
                });
                setMergeState({ phase: 'idle' });
              } catch (e) {
                setMergeError(e instanceof Error ? e.message : String(e));
              }
            }}
            onCancel={() => { setMergeError(null); setMergeState({ phase: 'selecting', srcId: mergeState.srcId }); }}
          />
        );
      })()}

      {/* Delete confirm modal */}
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
            } catch (e) {
              setDeleteError(e instanceof Error ? e.message : String(e));
            }
          }}
          onCancel={() => { setDeleteError(null); setDeleteTarget(null); }}
        />
      )}

      {/* Reset registry confirm modal */}
      {showResetModal && (
        <ResetRegistryModal
          isPending={resetRegistry.isPending}
          error={resetError}
          onConfirm={async () => {
            setResetError(null);
            try {
              await resetRegistry.mutateAsync();
              setShowResetModal(false);
            } catch (e) {
              setResetError(e instanceof Error ? e.message : String(e));
            }
          }}
          onCancel={() => { setResetError(null); setShowResetModal(false); }}
        />
      )}
    </div>
  );
}
