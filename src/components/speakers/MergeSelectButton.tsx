import type { Speaker } from '../../types/speaker';

export type MergeState =
  | { phase: 'idle' }
  | { phase: 'selecting'; srcId: number }
  | { phase: 'confirming'; srcId: number; dstId: number };

interface Props {
  speaker:    Speaker;
  mergeState: MergeState;
  onSelect:   (speakerId: number) => void;
  onCancel:   () => void;
  srcName:    string | null;   // Name of the src speaker when phase=selecting
}

export function MergeSelectButton({ speaker, mergeState, onSelect, onCancel, srcName }: Props) {
  const { phase } = mergeState;
  const isSrc = phase === 'selecting' && mergeState.srcId === speaker.speech_swift_id;

  if (phase === 'idle') {
    return (
      <button
        onClick={() => onSelect(speaker.speech_swift_id)}
        className="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded text-gray-700 border border-gray-200"
      >
        Select
      </button>
    );
  }

  if (phase === 'selecting') {
    if (isSrc) {
      return (
        <button
          onClick={onCancel}
          className="px-2 py-1 text-xs bg-red-100 hover:bg-red-200 rounded text-red-700 border border-red-200"
        >
          Cancel
        </button>
      );
    }
    const srcDisplayName = srcName ?? 'Unknown';
    return (
      <button
        onClick={() => onSelect(speaker.speech_swift_id)}
        className="px-2 py-1 text-xs bg-blue-100 hover:bg-blue-200 rounded text-blue-700 border border-blue-200 whitespace-nowrap"
      >
        Merge with {srcDisplayName}
      </button>
    );
  }

  // confirming — all disabled
  return (
    <button disabled className="px-2 py-1 text-xs bg-gray-50 rounded text-gray-400 border border-gray-200">
      Select
    </button>
  );
}
