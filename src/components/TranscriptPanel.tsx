import { useRef, useEffect, useCallback } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { SegmentCard } from './SegmentCard';
import type { Segment } from '../types/transcript';

interface Props {
  segments:          Segment[];
  isRecording:       boolean;
  debugMode:         boolean;
  segmentKinds:      Record<number, 'fast' | 'slow'>;
  replacedBySlowMap: Record<number, Segment[]>;
}

export function TranscriptPanel({ segments, isRecording, debugMode, segmentKinds, replacedBySlowMap }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const isAtTopRef   = useRef(true);

  // Display newest first.
  const reversed = [...segments].reverse();

  const virtualizer = useVirtualizer({
    count:            reversed.length,
    getScrollElement: () => containerRef.current,
    estimateSize:     () => 80,
    overscan:         5,
    measureElement:   (el) => el.getBoundingClientRect().height,
  });

  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    isAtTopRef.current = el.scrollTop < 64;
  }, []);

  // Auto-scroll: when a new segment arrives, scroll to the top (index 0 =
  // most recent) only if the user was already there.
  useEffect(() => {
    if (reversed.length === 0) return;
    if (isAtTopRef.current) {
      virtualizer.scrollToIndex(0, { behavior: 'smooth' });
    }
  }, [reversed.length, virtualizer]);

  if (segments.length === 0) {
    if (isRecording) {
      return (
        <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">
          Waiting for speech...
        </div>
      );
    }
    return null;
  }

  const totalSize    = virtualizer.getTotalSize();
  const virtualItems = virtualizer.getVirtualItems();

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      className="overflow-y-auto flex-1"
    >
      <div style={{ height: totalSize, position: 'relative' }}>
        {virtualItems.map((vItem) => (
          <div
            key={vItem.key}
            ref={virtualizer.measureElement}
            data-index={vItem.index}
            style={{
              position:  'absolute',
              top:       0,
              left:      0,
              right:     0,
              transform: `translateY(${vItem.start}px)`,
            }}
          >
            <SegmentCard
              segment={reversed[vItem.index]}
              debugMode={debugMode}
              kind={segmentKinds[reversed[vItem.index].id]}
              replacedSegments={replacedBySlowMap[reversed[vItem.index].id]}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
