import { useState } from 'react';
import { useSimilarSpeakers, useMergeSpeakers } from '../../hooks/useSpeakers';
import { AudioPlayer } from './AudioPlayer';
import type { Speaker, SimilarSpeaker } from '../../types/speaker';

function Spinner() {
  return <div className="w-3 h-3 border border-gray-300 border-t-gray-500 rounded-full animate-spin" />;
}

const DOT_COLORS = [
  'bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-yellow-400',
  'bg-pink-500', 'bg-indigo-500', 'bg-orange-500', 'bg-teal-500',
];
function dotColor(id: number) { return DOT_COLORS[id % DOT_COLORS.length]; }

function scoreBadgeClass(score: number) {
  if (score >= 0.85) return 'bg-green-100 text-green-800';
  if (score >= 0.60) return 'bg-amber-100 text-amber-800';
  return 'bg-red-100 text-red-800';
}

function scoreLabel(score: number) {
  return `${Math.round(score * 100)}% match`;
}

interface Props {
  selectedSpeaker: Speaker;
  onClose:         () => void;
  onDelete:        (speaker: Speaker) => void;
}

function SkeletonCard() {
  return (
    <div className="animate-pulse p-4 border-b border-gray-100">
      <div className="h-4 bg-gray-200 rounded w-16 mb-2" />
      <div className="h-3 bg-gray-200 rounded w-32 mb-3" />
      <div className="h-8 bg-gray-200 rounded w-full" />
    </div>
  );
}

interface CardProps {
  similar:         SimilarSpeaker;
  selectedSpeaker: Speaker;
  onMergeClick:    (target: Speaker) => void;
  onDelete:        (speaker: Speaker) => void;
}

function SimilarSpeakerCard({ similar, selectedSpeaker, onMergeClick, onDelete }: CardProps) {
  const { speaker } = similar;
  const name = speaker.display_name ?? `Unknown Speaker #${speaker.speech_swift_id}`;
  const isUnrecognized = speaker.display_name === null;

  return (
    <div className="p-4 border-b border-gray-100">
      <div className="flex items-center gap-2 mb-2">
        <span className={`text-xs font-semibold px-2 py-0.5 rounded-full ${scoreBadgeClass(similar.similarity_score)}`}>
          {scoreLabel(similar.similarity_score)}
        </span>
        {isUnrecognized && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-amber-100 text-amber-700">Unrecognized</span>
        )}
      </div>
      <div className="flex items-center gap-2 mb-2">
        <div className={`w-2 h-2 rounded-full flex-shrink-0 ${isUnrecognized ? 'bg-gray-400' : dotColor(speaker.speech_swift_id)}`} />
        <span className={`text-sm font-medium ${isUnrecognized ? 'italic text-gray-400' : 'text-gray-900'}`}>{name}</span>
        <span className="text-xs text-gray-400">{speaker.session_count} sessions</span>
      </div>
      <div className="flex items-center gap-2 mb-3">
        <AudioPlayer speechSwiftId={speaker.speech_swift_id} />
      </div>
      <div className="flex gap-2">
        <button
          onClick={() => onMergeClick(speaker)}
          className="flex-1 px-2 py-1.5 text-xs font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors"
        >
          Merge into {selectedSpeaker.display_name ?? 'selected'}
        </button>
        <button
          onClick={() => onDelete(speaker)}
          className="px-2 py-1.5 text-xs font-medium border border-red-300 text-red-600 hover:bg-red-50 rounded-md transition-colors"
        >
          Delete
        </button>
      </div>
    </div>
  );
}

export function SimilarSpeakersPanel({ selectedSpeaker, onClose, onDelete }: Props) {
  const [mergeTarget, setMergeTarget] = useState<Speaker | null>(null);
  const [mergeError, setMergeError] = useState<string | null>(null);
  const { data: similar = [], isLoading } = useSimilarSpeakers(selectedSpeaker.speech_swift_id);
  const mergeSpeakers = useMergeSpeakers();

  const selectedName = selectedSpeaker.display_name ?? `Unknown Speaker #${selectedSpeaker.speech_swift_id}`;
  const selectedDotClass = selectedSpeaker.display_name === null ? 'bg-gray-400' : dotColor(selectedSpeaker.speech_swift_id);

  async function handleConfirmMerge() {
    if (!mergeTarget) return;
    setMergeError(null);
    try {
      await mergeSpeakers.mutateAsync({ srcId: mergeTarget.speech_swift_id, dstId: selectedSpeaker.speech_swift_id });
      setMergeTarget(null);
    } catch (e) {
      setMergeError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="w-80 flex-shrink-0 flex flex-col border-l border-gray-200 bg-white overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 px-4 h-14 border-b border-gray-200 flex-shrink-0">
        <div className={`w-6 h-6 rounded-full flex-shrink-0 ${selectedDotClass}`} />
        <span className="text-sm font-semibold text-gray-900 flex-1 truncate">{selectedName}</span>
        <button onClick={onClose} className="text-gray-400 hover:text-gray-600 transition-colors text-lg leading-none">
          ×
        </button>
      </div>

      {/* Section label */}
      <div className="flex items-center gap-2 px-4 py-2 bg-gray-50 border-b border-gray-100 flex-shrink-0">
        <span className="text-xs font-semibold text-gray-500 uppercase tracking-wide flex-1">
          Most Similar Speakers
        </span>
        {isLoading && <Spinner />}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto">
        {/* Merge confirmation */}
        {mergeTarget && (
          <div className="m-3 p-4 bg-amber-50 border border-amber-200 rounded-lg">
            <p className="text-sm font-semibold text-amber-900 mb-1">Confirm Merge?</p>
            <p className="text-xs text-amber-800 mb-1">
              Merge <strong>{mergeTarget.display_name ?? `Unknown #${mergeTarget.speech_swift_id}`}</strong> → <strong>{selectedName}</strong>
            </p>
            <p className="text-xs text-amber-700 mb-3">All segments and sessions will be reassigned. This cannot be undone.</p>
            {mergeError && (
              <p className="text-xs text-red-600 bg-red-50 border border-red-200 rounded p-2 mb-2">{mergeError}</p>
            )}
            <div className="flex gap-2">
              <button
                onClick={handleConfirmMerge}
                disabled={mergeSpeakers.isPending}
                className="flex-1 px-3 py-1.5 text-xs font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-md disabled:opacity-50 transition-colors"
              >
                {mergeSpeakers.isPending ? 'Merging…' : 'Confirm Merge'}
              </button>
              <button
                onClick={() => { setMergeTarget(null); setMergeError(null); }}
                disabled={mergeSpeakers.isPending}
                className="px-3 py-1.5 text-xs font-medium border border-gray-300 text-gray-700 hover:bg-gray-50 rounded-md transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {/* Loading skeletons */}
        {isLoading && !mergeTarget && (
          <>
            <SkeletonCard />
            <SkeletonCard />
            <SkeletonCard />
          </>
        )}

        {/* Results */}
        {!isLoading && !mergeTarget && similar.length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
            <p className="text-sm font-medium text-gray-500 mb-1">No similar speakers found</p>
            <p className="text-xs text-gray-400">The similarity endpoint may not be available yet.</p>
          </div>
        )}

        {!isLoading && !mergeTarget && similar.map(s => (
          <SimilarSpeakerCard
            key={s.speaker.speech_swift_id}
            similar={s}
            selectedSpeaker={selectedSpeaker}
            onMergeClick={setMergeTarget}
            onDelete={onDelete}
          />
        ))}
      </div>

      {/* Footer */}
      <div className="border-t border-gray-200 p-3 flex flex-col gap-2 flex-shrink-0">
        <button
          onClick={() => onDelete(selectedSpeaker)}
          className="w-full px-3 py-1.5 text-xs font-medium border border-red-300 text-red-600 hover:bg-red-50 rounded-md transition-colors text-left"
        >
          Delete {selectedName}
        </button>
      </div>
    </div>
  );
}
