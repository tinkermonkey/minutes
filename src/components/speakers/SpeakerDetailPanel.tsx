import { useNavigate } from '@tanstack/react-router';
import { useSpeakerDetail } from '../../hooks/useSpeakers';
import { SpeakerNameField } from './SpeakerNameField';
import { AudioPlayer } from './AudioPlayer';
import type { Speaker } from '../../types/speaker';

const AVATAR_COLORS = [
  'bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-yellow-500',
  'bg-pink-500', 'bg-indigo-500', 'bg-orange-500', 'bg-teal-500',
];
function avatarColor(id: number) { return AVATAR_COLORS[id % AVATAR_COLORS.length]; }

function formatDate(ms: number) {
  return new Date(ms).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function formatDuration(ms: number | null) {
  if (!ms) return '—';
  return `${Math.floor(ms / 60000)} min`;
}

function formatTimestamp(ms: number) {
  const m = Math.floor(ms / 60000);
  const s = Math.floor((ms % 60000) / 1000);
  return `${m}:${String(s).padStart(2, '0')}`;
}

interface Props {
  speaker:       Speaker;
  onClose:       () => void;
  onFindSimilar: () => void;
  onDelete:      (speaker: Speaker) => void;
}

export function SpeakerDetailPanel({ speaker, onClose, onFindSimilar, onDelete }: Props) {
  const { data: detail, isLoading } = useSpeakerDetail(speaker.speech_swift_id);
  const navigate = useNavigate();

  const initial = speaker.display_name?.[0]?.toUpperCase() ?? '?';
  const isUnrecognized = speaker.display_name === null;

  return (
    <div className="w-[440px] flex-shrink-0 flex flex-col border-l border-gray-200 bg-white overflow-hidden">
      {/* Header */}
      <div className="flex items-start gap-3 p-4 border-b border-gray-200 flex-shrink-0">
        <div className={`w-12 h-12 rounded-full flex-shrink-0 flex items-center justify-center text-white text-lg font-semibold ${isUnrecognized ? 'bg-gray-400' : avatarColor(speaker.speech_swift_id)}`}>
          {initial}
        </div>
        <div className="flex-1 min-w-0 pt-1">
          <SpeakerNameField speaker={speaker} />
          {isUnrecognized && (
            <span className="text-xs px-1.5 py-0.5 rounded bg-amber-100 text-amber-700 mt-1 inline-block">
              Unrecognized
            </span>
          )}
        </div>
        <button onClick={onClose} className="text-gray-400 hover:text-gray-600 transition-colors mt-0.5 flex-shrink-0">
          <span className="text-lg leading-none">×</span>
        </button>
      </div>

      {/* Stats row */}
      <div className="flex gap-4 px-4 py-2 bg-gray-50 border-b border-gray-100 text-xs text-gray-500 flex-shrink-0">
        <span>{speaker.session_count} sessions</span>
        <span>First: {formatDate(speaker.first_seen_at)}</span>
        <span>Last: {formatDate(speaker.last_seen_at)}</span>
      </div>

      {/* Scrollable body */}
      <div className="flex-1 overflow-y-auto">
        {/* Recent Sessions */}
        <div className="px-4 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wide bg-gray-50 border-b border-gray-100">
          Recent Sessions
        </div>

        {isLoading && (
          <div className="animate-pulse px-4 py-2 space-y-2">
            {[1, 2, 3].map(i => <div key={i} className="h-8 bg-gray-200 rounded" />)}
          </div>
        )}

        {!isLoading && (detail?.recent_sessions.length ?? 0) === 0 && (
          <p className="px-4 py-3 text-xs text-gray-400">No sessions yet.</p>
        )}

        {!isLoading && detail?.recent_sessions.map(session => (
          <div
            key={session.id}
            className="flex items-center gap-2 px-4 py-2 border-b border-gray-100 hover:bg-gray-50 cursor-pointer transition-colors"
            onClick={() => navigate({ to: '/sessions/$sessionId', params: { sessionId: String(session.id) } })}
          >
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900 truncate">
                {session.label ?? formatDate(session.created_at)}
              </p>
              <p className="text-xs text-gray-400">{formatDuration(session.duration_ms)}</p>
            </div>
            <span className="flex-shrink-0 text-xs text-gray-500 bg-gray-100 rounded-full px-2 py-0.5">
              {session.segment_count} segments
            </span>
          </div>
        ))}

        {/* Recent Transcript */}
        {!isLoading && (
          <>
            <div className="px-4 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wide bg-gray-50 border-b border-gray-100 border-t border-t-gray-200">
              Recent Transcript
            </div>

            {(detail?.recent_segments.length ?? 0) === 0 && (
              <p className="px-4 py-3 text-xs text-gray-400">No confirmed segments yet.</p>
            )}

            {detail?.recent_segments.map(seg => (
              <div key={seg.id} className="flex gap-3 px-4 py-2 border-b border-gray-100">
                <span className="text-xs font-mono text-gray-400 w-10 flex-shrink-0 pt-0.5">
                  {formatTimestamp(seg.start_ms)}
                </span>
                <p className="text-sm text-gray-700 leading-relaxed">{seg.transcript_text}</p>
              </div>
            ))}

            {/* Voice Sample */}
            <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-100 border-t border-t-gray-200">
              <span className="text-xs text-gray-500 font-medium">Voice Sample</span>
              <AudioPlayer speechSwiftId={speaker.speech_swift_id} />
            </div>
          </>
        )}
      </div>

      {/* Footer */}
      <div className="flex gap-2 p-3 border-t border-gray-200 flex-shrink-0">
        <button
          onClick={onFindSimilar}
          className="flex-1 px-3 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors"
        >
          Find Similar &amp; Merge →
        </button>
        <button
          onClick={() => onDelete(speaker)}
          className="px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 rounded-md transition-colors"
        >
          Delete
        </button>
      </div>
    </div>
  );
}
