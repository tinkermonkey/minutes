import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient, useMutation } from '@tanstack/react-query';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useSpeechSwiftStatus } from '../hooks/useSpeechSwiftStatus';
import { useStartSession, useStopSession } from '../hooks/useSession';
import { useTauriEvent } from '../hooks/useTauriEvent';
import { useVadState } from '../hooks/useVadState';
import { RecordButton } from '../components/RecordButton';
import { SessionStatusBadge } from '../components/SessionStatusBadge';
import { AudioMeter } from '../components/AudioMeter';
import { AudioLevelGraph } from '../components/AudioLevelGraph';
import { PipelineEventLog } from '../components/PipelineEventLog';
import { TranscriptPanel } from '../components/TranscriptPanel';
import { NewSpeakerBanner } from '../components/NewSpeakerBanner';
import { SpeechSwiftErrorPanel } from '../components/SpeechSwiftErrorPanel';
import type {
  Segment,
  SpeakerNotification,
  SpeakerResolvedEvent,
  ChunkSentEvent,
  ChunkProcessedEvent,
  PipelineEntry,
  AccumulatorUpdatedEvent,
  SlowPathSentEvent,
  SlowPathDoneEvent,
} from '../types/transcript';

type SessionState =
  | { status: 'idle' }
  | { status: 'recording'; sessionId: number; startedAt: Date }
  | { status: 'stopping'; sessionId: number };

export function RecordRoute() {
  const { data: speechSwiftOk } = useSpeechSwiftStatus();
  const startSession = useStartSession();
  const stopSession  = useStopSession();
  const queryClient  = useQueryClient();

  const [sessionState, setSessionState] = useState<SessionState>({ status: 'idle' });
  const [segments, setSegments]         = useState<Segment[]>([]);
  const [showNewSpeakerBanner, setShowNewSpeakerBanner] = useState(false);
  const [elapsed, setElapsed]           = useState(0);
  const [pipelineEntries, setPipelineEntries] = useState<PipelineEntry[]>([]);
  const [accumulatorSecs,    setAccumulatorSecs]    = useState(0);
  const [accumulatorTrigger, setAccumulatorTrigger] = useState(30);

  const isRecording = sessionState.status === 'recording';
  const vadActive   = useVadState(isRecording);

  // Buffer for speaker_resolved events that arrive before segment_added has
  // applied the segment to state. Keyed by segment id.
  const pendingResolutions = useRef<Map<number, SpeakerResolvedEvent>>(new Map());

  const retryHealth = useMutation({
    mutationFn: (): Promise<boolean> => invoke('retry_health_check'),
    onSuccess: (reachable) => {
      queryClient.setQueryData(['speech_swift_status'], reachable);
    },
  });

  // Window title effect
  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();
    if (sessionState.status === 'recording') {
      appWindow.setTitle('Recording… — Minutes');
    } else {
      appWindow.setTitle('Minutes');
    }
    return () => { appWindow.setTitle('Minutes'); };
  }, [sessionState.status]);

  // Elapsed timer — ticks once per second while recording
  useEffect(() => {
    if (sessionState.status !== 'recording') return;
    const startedAt = sessionState.startedAt;
    const id = setInterval(() => {
      setElapsed(Date.now() - startedAt.getTime());
    }, 1000);
    return () => clearInterval(id);
  }, [sessionState]);

  useTauriEvent<Segment>('segment_added', payload => {
    setSegments(prev => {
      // Deduplicate: StrictMode double-subscription can fire the handler twice
      // for the same event in the brief window before the first subscription
      // is cleaned up.
      if (prev.some(s => s.id === payload.id)) return prev;

      const pending = pendingResolutions.current.get(payload.id);
      if (pending) {
        pendingResolutions.current.delete(payload.id);
        return [...prev, {
          ...payload,
          speaker_id:    pending.speaker_id,
          speaker_label: pending.speaker_label,
          display_name:  pending.display_name,
          status:        'confirmed' as const,
        }];
      }
      return [...prev, payload];
    });
  });

  useTauriEvent<SpeakerResolvedEvent>('speaker_resolved', payload => {
    setSegments(prev => {
      const seg = prev.find(s => s.id === payload.segment_id);
      if (!seg) {
        // Segment not yet applied to state — buffer the resolution.
        pendingResolutions.current.set(payload.segment_id, payload);
        return prev;
      }
      // Skip if already confirmed with a speaker (idempotent for double-fire).
      if (seg.status === 'confirmed' && seg.speaker_id === payload.speaker_id) {
        return prev;
      }
      return prev.map(s =>
        s.id === payload.segment_id
          ? { ...s,
              speaker_id:    payload.speaker_id,
              speaker_label: payload.speaker_label,
              display_name:  payload.display_name,
              status:        'confirmed' as const }
          : s
      );
    });
  });

  useTauriEvent<SpeakerNotification>('new_speaker', () => {
    setShowNewSpeakerBanner(true);
  });

  useTauriEvent<void>('speech_swift_unreachable', () => {
    queryClient.setQueryData(['speech_swift_status'], false);
  });

  useTauriEvent<ChunkSentEvent>('chunk_sent', payload => {
    setPipelineEntries(prev => [...prev, {
      kind:       'fast' as const,
      start_ms:   payload.start_ms,
      end_ms:     payload.end_ms,
      sent_at_ms: payload.sent_at_ms,
    }]);
  });

  useTauriEvent<ChunkProcessedEvent>('chunk_processed', payload => {
    setPipelineEntries(prev => prev.map(entry =>
      entry.kind === 'fast' && entry.start_ms === payload.start_ms
        ? { ...entry,
            response_ms:   payload.response_ms,
            word_count:    payload.word_count,
            speaker_count: payload.speaker_count }
        : entry
    ));
  });

  useTauriEvent<AccumulatorUpdatedEvent>('accumulator_updated', payload => {
    setAccumulatorSecs(payload.speech_secs);
    setAccumulatorTrigger(payload.trigger_secs);
  });

  useTauriEvent<SlowPathSentEvent>('slow_path_sent', payload => {
    setPipelineEntries(prev => [...prev, {
      kind:             'slow' as const,
      start_ms:         payload.start_ms,
      end_ms:           payload.end_ms,
      clip_speech_secs: payload.clip_speech_secs,
      sent_at_ms:       payload.sent_at_ms,
    }]);
  });

  useTauriEvent<SlowPathDoneEvent>('slow_path_done', payload => {
    setPipelineEntries(prev => prev.map(entry =>
      entry.kind === 'slow' && entry.start_ms === payload.start_ms
        ? { ...entry, response_ms: payload.response_ms, segment_count: payload.segment_count }
        : entry
    ));
  });

  async function handleStart() {
    const sessionId = await startSession.mutateAsync();
    setSessionState({ status: 'recording', sessionId, startedAt: new Date() });
    setSegments([]);
    setShowNewSpeakerBanner(false);
    setElapsed(0);
    setPipelineEntries([]);
    setAccumulatorSecs(0);
    setAccumulatorTrigger(30);
    pendingResolutions.current.clear();
  }

  async function handleStop() {
    if (sessionState.status !== 'recording') return;
    const { sessionId } = sessionState;
    setSessionState({ status: 'stopping', sessionId });
    await stopSession.mutateAsync(sessionId);
    setSessionState({ status: 'idle' });
    queryClient.invalidateQueries({ queryKey: ['segments', sessionId] });
  }

  if (speechSwiftOk === false) {
    return (
      <SpeechSwiftErrorPanel
        onRetry={() => retryHealth.mutate()}
        isRetrying={retryHealth.isPending}
      />
    );
  }

  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Controls row */}
      <div className="flex items-center gap-4">
        <RecordButton
          status={sessionState.status}
          disabled={!speechSwiftOk}
          onStart={handleStart}
          onStop={handleStop}
        />
        <SessionStatusBadge status={sessionState.status} elapsedMs={elapsed} />
        <AudioMeter active={isRecording} vadActive={vadActive} />
      </div>

      {/* New speaker banner */}
      {showNewSpeakerBanner && (
        <NewSpeakerBanner onDismiss={() => setShowNewSpeakerBanner(false)} />
      )}

      {/* Audio level graph — shown during and immediately after a recording */}
      {sessionState.status !== 'idle' && (
        <AudioLevelGraph active={isRecording} vadActive={vadActive} />
      )}

      {/* Pipeline event log — only shown during or after a recording */}
      {(sessionState.status !== 'idle' || pipelineEntries.length > 0) && (
        <div className="flex flex-col gap-1">
          <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
            Pipeline
          </h2>
          <PipelineEventLog
            entries={pipelineEntries}
            accumulatorSecs={accumulatorSecs}
            accumulatorTrigger={accumulatorTrigger}
          />
        </div>
      )}

      {/* Transcript panel */}
      <TranscriptPanel
        segments={segments}
        isRecording={isRecording}
      />
    </div>
  );
}
