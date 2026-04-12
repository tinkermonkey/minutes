import type { PipelineEntry } from '../types/transcript';

interface Props {
  entries: PipelineEntry[];
}

function fmtTime(unixMs: number): string {
  return new Date(unixMs).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function fmtPosition(startMs: number, endMs: number): string {
  const fmt = (ms: number) => {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    return `${m}:${String(s % 60).padStart(2, '0')}`;
  };
  return `${fmt(startMs)} – ${fmt(endMs)}`;
}

export function PipelineEventLog({ entries }: Props) {
  if (entries.length === 0) {
    return (
      <div className="text-xs text-gray-400 italic px-1">
        No chunks processed yet.
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-1 overflow-auto max-h-48">
      {/* Header */}
      <div className="grid grid-cols-[90px_110px_80px_60px_60px] gap-x-3 px-2 py-1 text-xs font-medium text-gray-500 border-b border-gray-100 sticky top-0 bg-white">
        <span>Time</span>
        <span>Position</span>
        <span>Response</span>
        <span>Words</span>
        <span>Speakers</span>
      </div>

      {/* Rows — newest first */}
      {[...entries].reverse().map(e => (
        <div
          key={e.start_ms}
          className="grid grid-cols-[90px_110px_80px_60px_60px] gap-x-3 px-2 py-1 text-xs text-gray-700 rounded hover:bg-gray-50"
        >
          <span className="font-mono text-gray-500">{fmtTime(e.sent_at_ms)}</span>
          <span className="font-mono">{fmtPosition(e.start_ms, e.end_ms)}</span>
          {e.response_ms !== undefined ? (
            <span className={e.response_ms > 2000 ? 'text-amber-600 font-medium' : ''}>
              {e.response_ms} ms
            </span>
          ) : (
            <span className="text-gray-400 animate-pulse">pending…</span>
          )}
          <span>{e.word_count ?? '—'}</span>
          <span>{e.speaker_count ?? '—'}</span>
        </div>
      ))}
    </div>
  );
}
