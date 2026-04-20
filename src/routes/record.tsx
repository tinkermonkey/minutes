import { useState, useMemo, useRef } from 'react';
import { useSpeechSwiftStatus } from '../hooks/useSpeechSwiftStatus';
import { useRecording } from '../contexts/RecordingContext';
import { AudioLevelGraph } from '../components/AudioLevelGraph';
import { PipelineEventLog } from '../components/PipelineEventLog';
import { TranscriptPanel } from '../components/TranscriptPanel';
import { NewSpeakerBanner } from '../components/NewSpeakerBanner';
import { SpeechSwiftErrorPanel } from '../components/SpeechSwiftErrorPanel';
import { SessionSpeakersSidebar } from '../components/SessionSpeakersSidebar';
import { ContinueSessionModal } from '../components/ContinueSessionModal';

// ─── Session Control Bar ───────────────────────────────────────────────────

interface IdleBarProps {
  newLabel: string;
  onLabelChange: (v: string) => void;
  onStart: () => void;
  onOpenContinue: () => void;
  isStarting: boolean;
  disabled: boolean;
}

function IdleBar({ newLabel, onLabelChange, onStart, onOpenContinue, isStarting, disabled }: IdleBarProps) {
  return (
    <div className="flex items-center gap-3 px-5 py-3 bg-white border-b border-gray-200 flex-shrink-0">
      <input
        type="text"
        value={newLabel}
        onChange={e => onLabelChange(e.target.value)}
        onKeyDown={e => { if (e.key === 'Enter' && !disabled && !isStarting) onStart(); }}
        placeholder="Name this session..."
        className="flex-1 min-w-0 text-sm rounded-lg border border-gray-200 px-3 py-2 text-gray-700 placeholder-gray-400 focus:outline-none focus:ring-1 focus:ring-blue-400"
      />
      <button
        type="button"
        onClick={onStart}
        disabled={disabled || isStarting}
        className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-white bg-green-600 hover:bg-green-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
      >
        {isStarting ? 'Starting...' : (
          <>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5,3 19,12 5,21" />
            </svg>
            Start Recording
          </>
        )}
      </button>
      <button
        type="button"
        onClick={onOpenContinue}
        className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium text-blue-600 border border-blue-300 hover:bg-blue-50 transition-colors"
      >
        Continue Session
        <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="9,18 15,12 9,6" />
        </svg>
      </button>
    </div>
  );
}

interface ActiveBarProps {
  sessionLabel: string;
  elapsedMs: number;
  speakerCount: number;
  isStopping: boolean;
  onStop: () => void;
  onRename: (label: string) => void;
}

function formatElapsed(ms: number): string {
  const totalSecs = Math.floor(ms / 1000);
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = totalSecs % 60;
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function ActiveBar({ sessionLabel, elapsedMs, speakerCount, isStopping, onStop, onRename }: ActiveBarProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  function startEdit() {
    setDraft(sessionLabel);
    setEditing(true);
    // Focus handled by autoFocus
  }

  function commitEdit() {
    const trimmed = draft.trim();
    if (trimmed) onRename(trimmed);
    setEditing(false);
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter') { e.preventDefault(); commitEdit(); }
    if (e.key === 'Escape') setEditing(false);
  }

  return (
    <div className="flex items-center gap-4 px-5 py-3 bg-white border-b border-gray-200 flex-shrink-0">
      {/* Recording indicator */}
      <span
        className="inline-block w-2.5 h-2.5 rounded-full bg-red-500 animate-pulse flex-shrink-0"
        title="Recording"
      />

      {/* Session name — inline edit */}
      {editing ? (
        <input
          ref={inputRef}
          autoFocus
          value={draft}
          onChange={e => setDraft(e.target.value)}
          onBlur={commitEdit}
          onKeyDown={handleKeyDown}
          className="flex-1 min-w-0 text-sm font-medium rounded border border-blue-400 px-2 py-1 text-gray-800 focus:outline-none focus:ring-1 focus:ring-blue-400"
          placeholder="Session name..."
        />
      ) : (
        <button
          type="button"
          onClick={startEdit}
          title="Click to rename"
          className="flex-1 min-w-0 text-left text-sm font-medium text-gray-800 hover:text-blue-600 truncate transition-colors"
        >
          {sessionLabel || 'Untitled Session'}
        </button>
      )}

      {/* Elapsed */}
      <span className="text-sm font-mono text-gray-600 tabular-nums flex-shrink-0">
        {formatElapsed(elapsedMs)}
      </span>

      {/* Speaker count chip */}
      {speakerCount > 0 && (
        <span className="inline-flex items-center gap-1 text-xs font-medium text-gray-500 bg-gray-100 rounded-full px-2.5 py-0.5 flex-shrink-0">
          <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="opacity-70">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
            <circle cx="9" cy="7" r="4" />
            <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
          </svg>
          {speakerCount}
        </span>
      )}

      {/* Stop button */}
      <button
        type="button"
        onClick={onStop}
        disabled={isStopping}
        className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-white bg-red-600 hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex-shrink-0"
      >
        {isStopping ? (
          'Stopping...'
        ) : (
          <>
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <rect x="3" y="3" width="18" height="18" />
            </svg>
            Stop
          </>
        )}
      </button>
    </div>
  );
}

// ─── Route ────────────────────────────────────────────────────────────────

export function RecordRoute() {
  const { data: speechSwiftOk } = useSpeechSwiftStatus();
  const {
    retryHealth,
    sessionState,
    segments,
    segmentKinds,
    replacedBySlowMap,
    pipelineEntries,
    showNewSpeakerBanner,
    setShowNewSpeakerBanner,
    vadActive,
    elapsed,
    sessionLabel,
    isStarting,
    handleStart,
    handleStop,
    handleResume,
    handleRenameSession,
  } = useRecording();

  const [debugMode, setDebugMode] = useState(false);
  const [newLabel, setNewLabel] = useState('');
  const [showContinuePicker, setShowContinuePicker] = useState(false);

  const isIdle      = sessionState.status === 'idle';
  const isRecording = sessionState.status === 'recording';
  const isStopping  = sessionState.status === 'stopping';

  const speakerCount = useMemo(() => {
    const ids = new Set<number>();
    for (const seg of segments) {
      if (seg.speaker_id != null) ids.add(seg.speaker_id);
    }
    return ids.size;
  }, [segments]);

  if (speechSwiftOk === false) {
    return (
      <SpeechSwiftErrorPanel
        onRetry={() => retryHealth.mutate()}
        isRetrying={retryHealth.isPending}
      />
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Session Control Bar */}
      {isIdle ? (
        <IdleBar
          newLabel={newLabel}
          onLabelChange={setNewLabel}
          onStart={() => handleStart(newLabel.trim() || undefined)}
          onOpenContinue={() => setShowContinuePicker(true)}
          isStarting={isStarting}
          disabled={!speechSwiftOk}
        />
      ) : (
        <ActiveBar
          sessionLabel={sessionLabel}
          elapsedMs={elapsed}
          speakerCount={speakerCount}
          isStopping={isStopping}
          onStop={handleStop}
          onRename={handleRenameSession}
        />
      )}

      {/* Main content area */}
      {isIdle ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <p className="text-lg font-medium text-gray-500">Ready to record</p>
            <p className="text-sm text-gray-400 mt-1">
              Name your session and press Start Recording, or continue a previous one.
            </p>
          </div>
        </div>
      ) : (
        <div className="flex flex-1 overflow-hidden">
          <div className="flex flex-col gap-4 p-6 flex-1 overflow-y-auto">
            {showNewSpeakerBanner && (
              <NewSpeakerBanner onDismiss={() => setShowNewSpeakerBanner(false)} />
            )}

            <AudioLevelGraph active={isRecording} vadActive={vadActive} />

            {(isRecording || pipelineEntries.length > 0) && (
              <div className="flex flex-col gap-1">
                <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
                  Pipeline
                </h2>
                <PipelineEventLog entries={pipelineEntries} />
              </div>
            )}

            <div className="flex flex-col flex-1 min-h-0 gap-1">
              {segments.length > 0 && (
                <div className="flex items-center justify-between px-1">
                  <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                    Transcript
                  </h2>
                  <label className="flex items-center gap-1.5 text-xs text-gray-500 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={debugMode}
                      onChange={e => setDebugMode(e.target.checked)}
                      className="w-3.5 h-3.5 accent-purple-600"
                    />
                    Debug
                  </label>
                </div>
              )}
              <TranscriptPanel
                segments={segments}
                isRecording={isRecording}
                debugMode={debugMode}
                segmentKinds={segmentKinds}
                replacedBySlowMap={replacedBySlowMap}
              />
            </div>
          </div>

          <SessionSpeakersSidebar />
        </div>
      )}

      {showContinuePicker && (
        <ContinueSessionModal
          onClose={() => setShowContinuePicker(false)}
          onResume={(sessionId) => {
            handleResume(sessionId);
            setShowContinuePicker(false);
          }}
        />
      )}
    </div>
  );
}
