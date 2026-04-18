import { speakerColor } from '../lib/speakerColor';
import type { Segment } from '../types/transcript';

interface Props {
  segment:           Segment;
  debugMode?:        boolean;
  kind?:             'fast' | 'slow';
  replacedSegments?: Segment[];
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

export function SegmentCard({ segment, debugMode, kind, replacedSegments }: Props) {
  const clipSecs = (segment.end_ms - segment.start_ms) / 1000;
  const hasReplacements = debugMode && kind === 'slow' && replacedSegments && replacedSegments.length > 0;

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-3 mb-2 flex items-start gap-3">
      {/* Speaker + debug badge */}
      <div className="flex flex-col gap-0.5">
        <SpeakerChip
          speakerId={segment.speaker_id}
          displayName={segment.display_name}
          speakerLabel={segment.speaker_label}
          status={segment.status}
        />
        {debugMode && kind && (
          <span className="text-xs font-mono text-gray-400">
            ({kind === 'fast' ? 'f' : 's'}) {clipSecs.toFixed(1)}s
          </span>
        )}
      </div>

      {/* Timestamp */}
      <span className="font-mono text-xs text-gray-400 whitespace-nowrap w-14 pt-0.5">
        {formatMs(segment.start_ms)}
      </span>

      {/* Transcript text */}
      <p className="text-sm text-gray-800 flex-1">
        {segment.transcript_text}
      </p>

      {/* Replaced fast entries column (debug only) */}
      {hasReplacements && (
        <div className="border-l border-dashed border-gray-300 pl-3 min-w-[180px] max-w-[280px]">
          <p className="text-xs font-semibold text-gray-400 mb-1 uppercase tracking-wide">Replaced</p>
          {replacedSegments!.map(r => (
            <div key={r.id} className="flex gap-2 mb-0.5">
              <span className="font-mono text-xs text-gray-400 whitespace-nowrap">
                {formatMs(r.start_ms)}
              </span>
              <span className="text-xs text-gray-500 leading-relaxed">
                {r.transcript_text}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
