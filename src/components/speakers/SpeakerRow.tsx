import type { Speaker } from '../../types/speaker';

const DOT_COLORS = [
  'bg-blue-500',
  'bg-green-500',
  'bg-purple-500',
  'bg-yellow-400',
  'bg-pink-500',
  'bg-indigo-500',
  'bg-orange-500',
  'bg-teal-500',
];

function dotColor(id: number) {
  return DOT_COLORS[id % DOT_COLORS.length];
}

function formatDate(ms: number) {
  return new Date(ms).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

interface Props {
  speaker:           Speaker;
  isSelected:        boolean;
  isChecked:         boolean;
  onSelect:          () => void;
  onCheck:           () => void;
  onOpenDetail:      () => void;
  recentTranscript?: string;
}

export function SpeakerRow({ speaker, isSelected, isChecked, onSelect, onCheck, onOpenDetail, recentTranscript }: Props) {
  const isUnrecognized = speaker.display_name === null;
  const rowBase = 'flex items-center h-12 px-6 gap-3 cursor-pointer border-b border-gray-100 transition-colors';
  const rowColor = isSelected
    ? 'bg-blue-50 border-l-2 border-l-blue-500'
    : isChecked
    ? 'bg-indigo-50 border-l-2 border-l-indigo-300'
    : isUnrecognized
    ? 'bg-amber-50 border-l-2 border-l-amber-400'
    : 'hover:bg-gray-50 border-l-2 border-l-transparent';

  const nameLabel = speaker.display_name ?? `Unknown Speaker #${speaker.speech_swift_id}`;
  const nameStyle = speaker.display_name
    ? 'text-sm font-medium text-gray-900 hover:text-blue-600 hover:underline text-left truncate'
    : 'text-sm italic text-gray-400 hover:text-blue-400 hover:underline text-left truncate';

  return (
    <div className={`${rowBase} ${rowColor}`} onClick={onSelect}>
      {/* Checkbox — click is isolated from the row's panel-open handler */}
      <div
        className="w-5 flex-shrink-0 flex items-center justify-center"
        onClick={e => { e.stopPropagation(); onCheck(); }}
      >
        <input
          type="checkbox"
          checked={isChecked}
          onChange={() => {}}
          className="w-3.5 h-3.5 rounded border-gray-300 text-blue-600 pointer-events-none"
        />
      </div>

      {/* Color dot */}
      <div className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${isUnrecognized ? 'bg-gray-400' : dotColor(speaker.speech_swift_id)}`} />

      {/* Speaker name */}
      <button
        className={`${nameStyle} min-w-[160px] max-w-[220px]`}
        onClick={e => { e.stopPropagation(); onOpenDetail(); }}
        title={nameLabel}
      >
        {nameLabel}
      </button>

      {/* Unrecognized badge */}
      {isUnrecognized && (
        <span className="flex-shrink-0 text-xs font-medium px-2 py-0.5 rounded-full bg-amber-100 text-amber-800">
          Unrecognized
        </span>
      )}

      {/* Session count chip */}
      <span className="flex-shrink-0 text-xs text-gray-500 bg-gray-100 rounded-full px-2 py-0.5">
        {speaker.session_count} {speaker.session_count === 1 ? 'session' : 'sessions'}
      </span>

      <span className="flex-1" />

      {/* Last seen */}
      <span className="flex-shrink-0 text-xs text-gray-400 w-28 text-right">
        {formatDate(speaker.last_seen_at)}
      </span>

      {/* Recent transcript snippet */}
      {recentTranscript && (
        <span className="text-xs text-gray-400 truncate max-w-xs hidden lg:block ml-4">
          {recentTranscript}
        </span>
      )}
    </div>
  );
}
