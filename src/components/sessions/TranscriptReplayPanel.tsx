import { useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { speakerColor } from '../../lib/speakerColor';
import { formatRelativeTime } from '../../lib/format';
import type { SegmentWithSpeaker } from '../../types/session';

interface Props {
  segments: SegmentWithSpeaker[];
}

function resolveLabel(seg: SegmentWithSpeaker): string {
  if (seg.status === 'pending') return 'Identifying...';
  if (seg.display_name) return seg.display_name;
  if (seg.speaker_id !== null) return `Speaker ${seg.speaker_id}`;
  return 'Unknown';
}

export function TranscriptReplayPanel({ segments }: Props) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: segments.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80,
    overscan: 5,
  });

  if (segments.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 text-gray-400 text-sm">
        No transcript available for this session.
      </div>
    );
  }

  return (
    <div ref={parentRef} className="flex-1 overflow-auto">
      <div
        style={{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }}
      >
        {virtualizer.getVirtualItems().map(virtualItem => {
          const seg = segments[virtualItem.index];
          const label = resolveLabel(seg);
          const isPending = seg.status === 'pending';
          const colorClass = seg.speaker_id !== null && !isPending
            ? speakerColor(seg.speaker_id)
            : 'bg-gray-100 text-gray-500';
          const isUnknown = seg.speaker_id === null && !isPending;

          return (
            <div
              key={virtualItem.key}
              data-index={virtualItem.index}
              ref={virtualizer.measureElement}
              style={{ position: 'absolute', top: 0, left: 0, width: '100%', transform: `translateY(${virtualItem.start}px)` }}
            >
              <div className="bg-white border border-gray-200 rounded-lg p-3 mb-2 flex items-start gap-3">
                <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium whitespace-nowrap flex-shrink-0 ${colorClass} ${isUnknown || isPending ? 'italic' : ''}`}>
                  {label}
                </span>
                <span className="text-xs text-gray-400 font-mono whitespace-nowrap w-14 flex-shrink-0 pt-0.5">
                  {formatRelativeTime(seg.start_ms)}
                </span>
                <span className="text-sm text-gray-900 flex-1">
                  {seg.transcript_text}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
