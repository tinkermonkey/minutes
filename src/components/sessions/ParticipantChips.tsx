import { speakerColor } from '../../lib/speakerColor';
import type { SessionParticipant } from '../../types/session';

interface Props {
  participants: SessionParticipant[];
  maxVisible?:  number;
}

export function ParticipantChips({ participants, maxVisible = 4 }: Props) {
  if (participants.length === 0) return <span className="text-gray-400">—</span>;

  const visible = participants.slice(0, maxVisible);
  const overflow = participants.length - maxVisible;

  return (
    <div className="flex flex-wrap gap-1">
      {visible.map(p => (
        <span
          key={p.speech_swift_id}
          className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${speakerColor(p.speech_swift_id)}`}
        >
          {p.display_name ?? 'Unknown'}
        </span>
      ))}
      {overflow > 0 && (
        <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
          +{overflow}
        </span>
      )}
    </div>
  );
}
