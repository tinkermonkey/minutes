import type { PipelineEntry, FastPathEntry, SlowPathEntry } from '../types/transcript';

interface Props {
  entries:            PipelineEntry[];
  accumulatorSecs:    number;
  accumulatorTrigger: number;
}

// Shared grid: badge | time | position | duration | response | detail | count
const GRID = 'grid grid-cols-[24px_90px_110px_48px_80px_60px_60px] gap-x-3';

function fmtTime(unixMs: number): string {
  return new Date(unixMs).toLocaleTimeString([], {
    hour:   '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function fmtDuration(startMs: number, endMs: number): string {
  return ((endMs - startMs) / 1000).toFixed(1) + 's';
}

function fmtPosition(startMs: number, endMs: number): string {
  const fmt = (ms: number) => {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    return `${m}:${String(s % 60).padStart(2, '0')}`;
  };
  return `${fmt(startMs)} – ${fmt(endMs)}`;
}

// ── Sub-components ────────────────────────────────────────────────────────────

function FastBadge() {
  return (
    <span className="inline-flex items-center justify-center w-5 h-5 rounded-full text-[10px] font-bold bg-blue-100 text-blue-700 select-none">
      F
    </span>
  );
}

function SlowBadge() {
  return (
    <span className="inline-flex items-center justify-center w-5 h-5 rounded-full text-[10px] font-bold bg-purple-100 text-purple-700 select-none">
      S
    </span>
  );
}

function FastRow({ e }: { e: FastPathEntry }) {
  return (
    <div className={`${GRID} px-2 py-1 text-xs text-gray-700 rounded hover:bg-gray-50`}>
      <FastBadge />
      <span className="font-mono text-gray-500">{fmtTime(e.sent_at_ms)}</span>
      <span className="font-mono">{fmtPosition(e.start_ms, e.end_ms)}</span>
      <span className="font-mono text-gray-500">{fmtDuration(e.start_ms, e.end_ms)}</span>
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
  );
}

function SlowRow({ e }: { e: SlowPathEntry }) {
  return (
    <div className={`${GRID} px-2 py-1 text-xs text-gray-700 rounded hover:bg-gray-50`}>
      <SlowBadge />
      <span className="font-mono text-gray-500">{fmtTime(e.sent_at_ms)}</span>
      <span className="font-mono">{fmtPosition(e.start_ms, e.end_ms)}</span>
      <span className="font-mono text-gray-500">{fmtDuration(e.start_ms, e.end_ms)}</span>
      {e.response_ms !== undefined ? (
        <span className={e.response_ms > 5000 ? 'text-amber-600 font-medium' : ''}>
          {e.response_ms} ms
        </span>
      ) : (
        <span className="text-gray-400 animate-pulse">pending…</span>
      )}
      <span>{e.clip_speech_secs.toFixed(1)}s</span>
      <span>{e.segment_count ?? '—'}</span>
    </div>
  );
}

// ── Accumulator fill bar ──────────────────────────────────────────────────────

interface AccumulatorBarProps {
  secs:    number;
  trigger: number;
}

function AccumulatorBar({ secs, trigger }: AccumulatorBarProps) {
  if (trigger <= 0) return null;
  const pct     = Math.min(1, secs / trigger) * 100;
  const nearFull = pct >= 90;
  const fillColor = nearFull ? 'bg-amber-400' : 'bg-blue-400';

  return (
    <div className="flex items-center gap-2 px-2 py-1">
      <span className="text-xs text-gray-500 w-[72px] shrink-0">Accumulator</span>
      <div className="flex-1 h-2 rounded-full bg-gray-100 overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-300 ${fillColor}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-xs text-gray-400 font-mono w-[72px] text-right shrink-0">
        {secs.toFixed(1)}s / {trigger.toFixed(1)}s
      </span>
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export function PipelineEventLog({ entries, accumulatorSecs, accumulatorTrigger }: Props) {
  return (
    <div className="flex flex-col gap-0">
      {/* Accumulator fill bar — always rendered when trigger is set */}
      <AccumulatorBar secs={accumulatorSecs} trigger={accumulatorTrigger} />

      {entries.length === 0 ? (
        <div className="text-xs text-gray-400 italic px-2 py-1">
          No chunks processed yet.
        </div>
      ) : (
        <div className="flex flex-col gap-1 overflow-auto max-h-48">
          {/* Header */}
          <div className={`${GRID} px-2 py-1 text-xs font-medium text-gray-500 border-b border-gray-100 sticky top-0 bg-white`}>
            <span />
            <span>Time</span>
            <span>Position</span>
            <span>Len</span>
            <span>Response</span>
            <span>Detail</span>
            <span>Count</span>
          </div>

          {/* Rows — newest first */}
          {[...entries].reverse().map(e =>
            e.kind === 'fast'
              ? <FastRow key={`fast-${e.start_ms}`} e={e} />
              : <SlowRow key={`slow-${e.start_ms}`} e={e} />
          )}
        </div>
      )}
    </div>
  );
}
