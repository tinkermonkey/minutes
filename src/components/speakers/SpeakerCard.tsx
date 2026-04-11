import { speakerColor } from '../../lib/speakerColor';
import { SpeakerNameField } from './SpeakerNameField';
import { AudioPlayer } from './AudioPlayer';
import { MergeSelectButton, type MergeState } from './MergeSelectButton';
import type { Speaker } from '../../types/speaker';

interface Props {
  speaker:        Speaker;
  mergeState:     MergeState;
  onMergeSelect:  (id: number) => void;
  onMergeCancel:  () => void;
  onDeleteClick:  () => void;
  srcSpeakerName: string | null;
  isRecording:    boolean;
}

function formatDate(ms: number): string {
  return new Date(ms).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

const BORDER_COLOR_MAP: Record<string, string> = {
  'bg-blue-100':   'border-l-blue-400',
  'bg-green-100':  'border-l-green-400',
  'bg-purple-100': 'border-l-purple-400',
  'bg-yellow-100': 'border-l-yellow-400',
  'bg-pink-100':   'border-l-pink-400',
  'bg-indigo-100': 'border-l-indigo-400',
  'bg-orange-100': 'border-l-orange-400',
  'bg-teal-100':   'border-l-teal-400',
};

export function SpeakerCard({
  speaker, mergeState, onMergeSelect, onMergeCancel, onDeleteClick, srcSpeakerName, isRecording,
}: Props) {
  // Use speech_swift_id for color to match chip colors in transcript view
  const colorClasses = speakerColor(speaker.speech_swift_id);
  const bgClass = colorClasses.split(' ')[0];
  const borderColor = BORDER_COLOR_MAP[bgClass] ?? 'border-l-gray-400';

  return (
    <div className={`bg-white rounded-lg border border-gray-200 border-l-4 ${borderColor} p-4`}>
      {/* Row 1: Name */}
      <div className="mb-2">
        <SpeakerNameField speaker={speaker} />
      </div>

      {/* Row 2: Meta */}
      <div className="text-xs text-gray-500 mb-3">
        First seen: {formatDate(speaker.first_seen_at)} ·{' '}
        Last seen: {formatDate(speaker.last_seen_at)} ·{' '}
        {speaker.session_count} {speaker.session_count === 1 ? 'session' : 'sessions'}
      </div>

      {/* Row 3: Actions */}
      <div className="flex items-center gap-2">
        <AudioPlayer speechSwiftId={speaker.speech_swift_id} />
        <MergeSelectButton
          speaker={speaker}
          mergeState={mergeState}
          onSelect={onMergeSelect}
          onCancel={onMergeCancel}
          srcName={srcSpeakerName}
        />
        <button
          onClick={onDeleteClick}
          disabled={isRecording}
          title={isRecording ? 'Cannot delete a speaker during an active session' : 'Delete speaker'}
          className="px-2 py-1 text-xs bg-red-50 hover:bg-red-100 rounded text-red-700 border border-red-200 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Delete
        </button>
      </div>
    </div>
  );
}
