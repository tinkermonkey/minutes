import { speakerColor } from '../lib/speakerColor';
import type { Segment } from '../types/transcript';

interface Props {
  segment: Segment;
}

function formatMs(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

function SpeakerChip({ speakerId, displayName, speakerLabel, status }: {
  speakerId:    number | null;
  displayName:  string | null;
  speakerLabel: string | null;
  status:       'pending' | 'confirmed';
}) {
  if (status === 'pending') {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-500 whitespace-nowrap italic">
        Identifying...
      </span>
    );
  }
  const colorClasses = speakerColor(speakerId);
  const label = displayName ?? speakerLabel ?? 'Unknown';
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium whitespace-nowrap ${colorClasses}`}>
      {label}
    </span>
  );
}

export function SegmentCard({ segment }: Props) {
  return (
    <div className="bg-white border border-gray-200 rounded-lg p-3 mb-2 flex items-start gap-3">
      <SpeakerChip
        speakerId={segment.speaker_id}
        displayName={segment.display_name}
        speakerLabel={segment.speaker_label}
        status={segment.status}
      />
      <span className="font-mono text-xs text-gray-400 whitespace-nowrap w-14 pt-0.5">
        {formatMs(segment.start_ms)}
      </span>
      <p className="text-sm text-gray-800 flex-1">
        {segment.transcript_text}
      </p>
    </div>
  );
}
