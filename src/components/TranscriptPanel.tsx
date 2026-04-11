import { useRef, useEffect } from 'react';
import { SegmentCard } from './SegmentCard';
import type { Segment } from '../types/transcript';

interface Props {
  segments:    Segment[];
  isRecording: boolean;
}

export function TranscriptPanel({ segments, isRecording }: Props) {
  const containerRef  = useRef<HTMLDivElement>(null);
  const bottomRef     = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);

  const handleScroll = () => {
    const el = containerRef.current;
    if (!el) return;
    isAtBottomRef.current =
      el.scrollHeight - el.scrollTop - el.clientHeight < 64;
  };

  useEffect(() => {
    if (isAtBottomRef.current) {
      bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [segments.length]);

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

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      className="overflow-y-auto flex-1"
    >
      {segments.map(segment => (
        <SegmentCard key={segment.id} segment={segment} />
      ))}
      <div ref={bottomRef} />
    </div>
  );
}
