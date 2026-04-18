import { useState, useRef, useEffect } from 'react';
import type { PipelineEntry, FastPathEntry, SlowPathEntry } from '../types/transcript';
import { useTauriEvent } from '../hooks/useTauriEvent';

interface Props {
  entries: PipelineEntry[];
}

// Shared grid: badge | time | position | duration | response | detail | best score
const GRID = 'grid grid-cols-[24px_90px_110px_48px_72px_60px_52px] gap-x-3';

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

function fmtSpeed(clipMs: number, responseMs: number): string {
  if (responseMs <= 0) return '—';
  return (clipMs / responseMs).toFixed(1) + 'x';
}

function fmtBestScore(score: number | null | undefined): string {
  if (score == null) return '—';
  return score.toFixed(2);
}

function FastRow({ e }: { e: FastPathEntry }) {
  const clipMs = e.end_ms - e.start_ms;
  const recognized = e.best_score != null;
  return (
    <div className={`${GRID} px-2 py-1 text-xs text-gray-700 rounded ${recognized ? 'bg-green-50 hover:bg-green-100' : 'hover:bg-gray-50'}`}>
      <FastBadge />
      <span className="font-mono text-gray-500">{fmtTime(e.sent_at_ms)}</span>
      <span className="font-mono">{fmtPosition(e.start_ms, e.end_ms)}</span>
      <span className="font-mono text-gray-500">{fmtDuration(e.start_ms, e.end_ms)}</span>
      {e.response_ms !== undefined ? (
        <span className={e.response_ms > 2000 ? 'text-amber-600 font-medium' : ''}>
          {e.response_ms} ms
        </span>
      ) : (
        <span className="text-gray-400 animate-pulse">—</span>
      )}
      <span className="font-mono text-gray-500">
        {e.response_ms !== undefined ? fmtSpeed(clipMs, e.response_ms) : '—'}
      </span>
      <span className="font-mono">{fmtBestScore(e.best_score)}</span>
    </div>
  );
}

function SlowRow({ e }: { e: SlowPathEntry }) {
  const clipMs = e.clip_speech_secs * 1000;
  const recognized = e.segment_count != null && e.segment_count > 0;
  return (
    <div className={`${GRID} px-2 py-1 text-xs text-gray-700 rounded ${recognized ? 'bg-green-50 hover:bg-green-100' : 'hover:bg-gray-50'}`}>
      <SlowBadge />
      <span className="font-mono text-gray-500">{fmtTime(e.sent_at_ms)}</span>
      <span className="font-mono">{fmtPosition(e.start_ms, e.end_ms)}</span>
      <span className="font-mono text-gray-500">{fmtDuration(e.start_ms, e.end_ms)}</span>
      {e.response_ms !== undefined ? (
        <span className={e.response_ms > 5000 ? 'text-amber-600 font-medium' : ''}>
          {e.response_ms} ms
        </span>
      ) : (
        <span className="text-gray-400 animate-pulse">—</span>
      )}
      <span className="font-mono text-gray-500">
        {e.response_ms !== undefined ? fmtSpeed(clipMs, e.response_ms) : '—'}
      </span>
      <span className="font-mono">{fmtBestScore(e.best_score)}</span>
    </div>
  );
}

// ── Accumulator fill bars ─────────────────────────────────────────────────────
//
// Design: AccumulatorBar is self-contained. It listens to accumulator and VAD
// events directly via refs (zero re-renders per event) and runs a 50 ms ticker
// that drives the display state. While VAD is active, extra seconds are
// estimated from wall-clock time so the bars fill smoothly during continuous
// speech — even though the Rust side only emits real values after 500 ms of
// trailing silence. When VAD goes inactive the bar freezes at the last estimate
// so there is no snap-back before the real chunk arrives. When a real event
// fires it snaps to truth.

interface AccumulatorBarProps {
  /** Reset key — pass the current session id (or null when idle). */
  sessionId: number | null;
}

export function AccumulatorBar({ sessionId }: AccumulatorBarProps) {
  // Mutable state — updated synchronously in event handlers, never cause renders.
  const fastSecsRef        = useRef(0);
  const lastFastEventAt    = useRef(Date.now());
  const fastTriggerRef     = useRef(2);
  const slowSecsRef        = useRef(0);
  const lastSlowEventAt    = useRef(Date.now());
  const slowTriggerRef     = useRef(10);
  const vadActiveSinceRef  = useRef<number | null>(null);
  // Frozen display values, written by the ticker, read by the VAD-stop handler.
  const lastDisplayFastRef = useRef(0);
  const lastDisplaySlowRef = useRef(0);

  // Display state — the only thing that drives re-renders.
  const [displayFast, setDisplayFast] = useState(0);
  const [displaySlow, setDisplaySlow] = useState(0);
  const [fastTrigger,  setFastTrigger]  = useState(2);
  const [slowTrigger,  setSlowTrigger]  = useState(10);

  // Reset all refs and display when a new session starts.
  useEffect(() => {
    const now = Date.now();
    fastSecsRef.current       = 0;
    lastFastEventAt.current   = now;
    slowSecsRef.current       = 0;
    lastSlowEventAt.current   = now;
    vadActiveSinceRef.current = null;
    setDisplayFast(0);
    setDisplaySlow(0);
  }, [sessionId]);

  // Real accumulator events — update refs only, no state change here.
  useTauriEvent<{ speech_secs: number; trigger_secs: number }>(
    'fast_accumulator_updated',
    ({ speech_secs, trigger_secs }) => {
      fastSecsRef.current     = speech_secs;
      lastFastEventAt.current = Date.now();
      setFastTrigger(t => t !== trigger_secs ? trigger_secs : t);
      fastTriggerRef.current  = trigger_secs;
    },
  );

  useTauriEvent<{ speech_secs: number; trigger_secs: number }>(
    'accumulator_updated',
    ({ speech_secs, trigger_secs }) => {
      slowSecsRef.current     = speech_secs;
      lastSlowEventAt.current = Date.now();
      setSlowTrigger(t => t !== trigger_secs ? trigger_secs : t);
      slowTriggerRef.current  = trigger_secs;
    },
  );

  // VAD state transitions — drive the estimation window.
  useTauriEvent<boolean>('vad_state', active => {
    if (active) {
      vadActiveSinceRef.current = Date.now();
    } else {
      // Freeze the bars at the last ticker estimate when speech stops.
      // The chunker still has ~500 ms of buffered audio; the real chunk
      // (and the accurate accumulator event) will arrive shortly after.
      fastSecsRef.current       = lastDisplayFastRef.current;
      lastFastEventAt.current   = Date.now();
      slowSecsRef.current       = lastDisplaySlowRef.current;
      lastSlowEventAt.current   = Date.now();
      vadActiveSinceRef.current = null;
    }
  });

  // 50 ms ticker — the only place where display state is written.
  useEffect(() => {
    const id = setInterval(() => {
      const now        = Date.now();
      const activeSince = vadActiveSinceRef.current;

      // During speech, estimate additional accumulator fill from wall-clock
      // time since whichever came later: VAD activation or the last real event.
      // This correctly handles the case where the fast acc drains mid-speech.
      const extraFast = activeSince != null
        ? Math.max(0, now - Math.max(activeSince, lastFastEventAt.current)) / 1000
        : 0;
      const extraSlow = activeSince != null
        ? Math.max(0, now - Math.max(activeSince, lastSlowEventAt.current)) / 1000
        : 0;

      const nextFast = Math.min(fastSecsRef.current + extraFast, fastTriggerRef.current);
      const nextSlow = Math.min(slowSecsRef.current + extraSlow, slowTriggerRef.current);

      lastDisplayFastRef.current = nextFast;
      lastDisplaySlowRef.current = nextSlow;
      setDisplayFast(nextFast);
      setDisplaySlow(nextSlow);
    }, 50);
    return () => clearInterval(id);
  }, []); // stable — reads only refs

  const fastPct = fastTrigger > 0 ? Math.min(1, displayFast / fastTrigger) * 100 : 0;
  const slowPct = slowTrigger > 0 ? Math.min(1, displaySlow / slowTrigger) * 100 : 0;

  return (
    <div className="flex flex-1 items-center gap-2">
      <span className="text-xs text-gray-500 shrink-0">Accumulator</span>
      <div className="flex-1 flex flex-col gap-[3px]">
        {/* Fast bar — blue → amber when near full */}
        <div className="h-[5px] rounded-full bg-gray-100 overflow-hidden">
          <div
            className={`h-full rounded-full ${fastPct >= 90 ? 'bg-amber-400' : 'bg-blue-400'}`}
            style={{ width: `${fastPct}%`, transition: 'width 50ms linear' }}
          />
        </div>
        {/* Slow bar — purple → amber when near full */}
        <div className="h-[5px] rounded-full bg-gray-100 overflow-hidden">
          <div
            className={`h-full rounded-full ${slowPct >= 90 ? 'bg-amber-500' : 'bg-purple-400'}`}
            style={{ width: `${slowPct}%`, transition: 'width 50ms linear' }}
          />
        </div>
      </div>
      <div className="text-xs text-gray-400 font-mono text-right shrink-0 leading-tight">
        <div>{fastTrigger.toFixed(0)}s</div>
        <div>{slowTrigger.toFixed(0)}s</div>
      </div>
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export function PipelineEventLog({ entries }: Props) {
  return (
    <div className="flex flex-col gap-0">
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
            <span>Speed</span>
            <span>Best score</span>
          </div>

          {/* Rows — newest first.
               In-flight (no response_ms yet): always show so the user sees the
               request immediately. Completed: hide noise-only entries. */}
          {[...entries].reverse().filter(e =>
            e.response_ms === undefined ||
            (e.kind === 'fast'
              ? e.best_score != null
              : e.segment_count != null && e.segment_count > 0)
          ).map(e =>
            e.kind === 'fast'
              ? <FastRow key={`fast-${e.start_ms}`} e={e} />
              : <SlowRow key={`slow-${e.start_ms}`} e={e} />
          )}
        </div>
      )}
    </div>
  );
}
