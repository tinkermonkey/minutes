import { ParticipantChips } from './ParticipantChips';
import { SourceBadge } from './SourceBadge';
import { formatDate, formatTime, formatDuration } from '../../lib/format';
import type { Session } from '../../types/session';

interface Props {
  session: Session;
  onBack:  () => void;
}

export function SessionDetailHeader({ session, onBack }: Props) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-3 flex-wrap">
        <button
          onClick={onBack}
          className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded-lg text-gray-700"
        >
          ← Sessions
        </button>
        <h2 className="text-xl font-semibold text-gray-900">
          {formatDate(session.created_at)} · {formatTime(session.created_at)}
        </h2>
        <span className="px-2 py-0.5 text-xs bg-gray-100 rounded text-gray-600">
          {formatDuration(session.duration_ms)}
        </span>
        <SourceBadge source={session.source} />
      </div>
      <ParticipantChips participants={session.participants} maxVisible={Infinity} />
    </div>
  );
}
