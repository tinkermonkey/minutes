import { createContext, useContext, useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient, useMutation } from '@tanstack/react-query';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useStartSession, useStopSession } from '../hooks/useSession';
import { useTauriEvent } from '../hooks/useTauriEvent';
import { useVadState } from '../hooks/useVadState';
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

export type SessionState =
  | { status: 'idle' }
  | { status: 'recording'; sessionId: number; startedAt: Date }
  | { status: 'stopping'; sessionId: number };

interface RecordingContextValue {
  sessionState: SessionState;
  language: 'english' | 'auto';
  setLanguage: (lang: 'english' | 'auto') => void;
  segments: Segment[];
  elapsed: number;
  pipelineEntries: PipelineEntry[];
  accumulatorSecs: number;
  accumulatorTrigger: number;
  showNewSpeakerBanner: boolean;
  setShowNewSpeakerBanner: (show: boolean) => void;
  vadActive: boolean;
  handleStart: () => Promise<void>;
  handleStop: () => Promise<void>;
  isStarting: boolean;
  retryHealth: { mutate: () => void; isPending: boolean };
}

const RecordingContext = createContext<RecordingContextValue | null>(null);

export function useRecording(): RecordingContextValue {
  const ctx = useContext(RecordingContext);
  if (!ctx) {
    throw new Error('useRecording must be used within a RecordingProvider');
  }
  return ctx;
}

export function RecordingProvider({ children }: { children: React.ReactNode }) {
  const queryClient  = useQueryClient();
  const startSession = useStartSession();
  const stopSession  = useStopSession();

  const [sessionState, setSessionState] = useState<SessionState>({ status: 'idle' });
  const [language, setLanguage]         = useState<'english' | 'auto'>('english');
  const [segments, setSegments]         = useState<Segment[]>([]);
  const [showNewSpeakerBanner, setShowNewSpeakerBanner] = useState(false);
  const [elapsed, setElapsed]           = useState(0);
  const [pipelineEntries, setPipelineEntries] = useState<PipelineEntry[]>([]);
  const [accumulatorSecs,    setAccumulatorSecs]    = useState(0);
  const [accumulatorTrigger, setAccumulatorTrigger] = useState(10);

  const vadActive = useVadState(sessionState.status === 'recording');

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
      appWindow.setTitle('Recording\u2026 \u2014 Minutes');
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
      if (!payload.transcript_text?.trim()) return prev;
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
        pendingResolutions.current.set(payload.segment_id, payload);
        return prev;
      }
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
    const sessionId = await startSession.mutateAsync(language);
    setSessionState({ status: 'recording', sessionId, startedAt: new Date() });
    setSegments([]);
    setShowNewSpeakerBanner(false);
    setElapsed(0);
    setPipelineEntries([]);
    setAccumulatorSecs(0);
    setAccumulatorTrigger(10);
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

  const value: RecordingContextValue = {
    sessionState,
    language,
    setLanguage,
    segments,
    elapsed,
    pipelineEntries,
    accumulatorSecs,
    accumulatorTrigger,
    showNewSpeakerBanner,
    setShowNewSpeakerBanner,
    vadActive,
    handleStart,
    handleStop,
    isStarting: startSession.isPending,
    retryHealth: { mutate: () => retryHealth.mutate(), isPending: retryHealth.isPending },
  };

  return (
    <RecordingContext.Provider value={value}>
      {children}
    </RecordingContext.Provider>
  );
}
