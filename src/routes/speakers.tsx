import { useState } from 'react';
import { useSpeakers, useMergeSpeakers, useDeleteSpeaker } from '../hooks/useSpeakers';
import { SpeakerCard } from '../components/speakers/SpeakerCard';
import { MergeConfirmModal } from '../components/speakers/MergeConfirmModal';
import { DeleteConfirmModal } from '../components/speakers/DeleteConfirmModal';
import { SpeakerCardSkeleton } from '../components/speakers/SpeakerCardSkeleton';
import { QueryError } from '../components/QueryError';
import type { Speaker } from '../types/speaker';
import type { MergeState } from '../components/speakers/MergeSelectButton';

export function SpeakersRoute() {
  const { data: speakers = [], isLoading, isError, error, refetch } = useSpeakers();
  const [mergeState, setMergeState] = useState<MergeState>({ phase: 'idle' });
  const [deleteTarget, setDeleteTarget] = useState<Speaker | null>(null);
  const mergeSpeakers = useMergeSpeakers();
  const deleteSpeaker = useDeleteSpeaker();

  const isRecording = false;

  function handleMergeSelect(selectedId: number) {
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
    return src?.display_name ?? null;
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
            onConfirm={async () => {
              await mergeSpeakers.mutateAsync({
                srcId: mergeState.srcId,
                dstId: mergeState.dstId,
              });
              setMergeState({ phase: 'idle' });
            }}
            onCancel={() => setMergeState({ phase: 'selecting', srcId: mergeState.srcId })}
          />
        );
      })()}

      {/* Delete confirm modal */}
      {deleteTarget && (
        <DeleteConfirmModal
          speaker={deleteTarget}
          isPending={deleteSpeaker.isPending}
          onConfirm={async () => {
            await deleteSpeaker.mutateAsync(deleteTarget.speech_swift_id);
            setDeleteTarget(null);
          }}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  );
}
