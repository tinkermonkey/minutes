import { createContext, useContext, useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient, useMutation } from '@tanstack/react-query';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useStartSession, useStopSession } from '../hooks/useSession';
import { useTauriEvent } from '../hooks/useTauriEvent';
import { useVadState } from '../hooks/useVadState';
import { useMicState } from '../hooks/useMicState';
import type {
  Segment,
  SpeakerNotification,
  SegmentsReplacedEvent,
  ChunkSentEvent,
  ChunkProcessedEvent,
  PipelineEntry,
  AccumulatorUpdatedEvent,
  FastAccumulatorUpdatedEvent,
  SlowPathSentEvent,
  SlowPathDoneEvent,
} from '../types/transcript';
import type {
  SpeakerRenamedEvent,
  SpeakersMergedEvent,
  SpeakerDeletedEvent,
} from '../types/speaker';

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
  fastAccumulatorSecs: number;
  fastAccumulatorTrigger: number;
  segmentKinds:      Record<number, 'fast' | 'slow'>;
  replacedBySlowMap: Record<number, Segment[]>;
  showNewSpeakerBanner: boolean;
  setShowNewSpeakerBanner: (show: boolean) => void;
  vadActive: boolean;
  micActive: boolean;
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
  const [segmentKinds,      setSegmentKinds]      = useState<Record<number, 'fast' | 'slow'>>({});
  const [replacedBySlowMap, setReplacedBySlowMap] = useState<Record<number, Segment[]>>({});
  const latestSegmentsRef = useRef<Segment[]>([]);
  const [pipelineEntries, setPipelineEntries] = useState<PipelineEntry[]>([]);
  const [accumulatorSecs,        setAccumulatorSecs]        = useState(0);
  const [accumulatorTrigger,     setAccumulatorTrigger]     = useState(10);
  const [fastAccumulatorSecs,    setFastAccumulatorSecs]    = useState(0);
  const [fastAccumulatorTrigger, setFastAccumulatorTrigger] = useState(2);

  const vadActive = useVadState(sessionState.status === 'recording');
  const micActive = useMicState();

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
    if (!payload.transcript_text?.trim()) return;
    setSegments(prev => {
      if (prev.some(s => s.id === payload.id)) return prev;
      const next = [...prev, payload];
      latestSegmentsRef.current = next;
      return next;
    });
    setSegmentKinds(prev => ({ ...prev, [payload.id]: 'fast' }));
  });

  useTauriEvent<SegmentsReplacedEvent>('segments_replaced', payload => {
    const removedSet  = new Set(payload.removed_ids);
    const removedSegs = latestSegmentsRef.current.filter(s => removedSet.has(s.id));
    const incoming    = payload.added.filter(s => s.transcript_text?.trim());

    setSegments(prev => {
      const kept = prev.filter(s => !removedSet.has(s.id));
      const next = [...kept, ...incoming];
      latestSegmentsRef.current = next;
      return next;
    });

    if (incoming.length > 0) {
      setSegmentKinds(prev => {
        const updates: Record<number, 'fast' | 'slow'> = {};
        for (const s of incoming) updates[s.id] = 'slow';
        return { ...prev, ...updates };
      });

      if (removedSegs.length > 0) {
        setReplacedBySlowMap(prev => {
          const updates: Record<number, Segment[]> = {};
          for (const s of incoming) updates[s.id] = removedSegs;
          return { ...prev, ...updates };
        });
      }
    }
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
            speaker_count: payload.speaker_count,
            best_score:    payload.best_score }
        : entry
    ));
  });

  useTauriEvent<AccumulatorUpdatedEvent>('accumulator_updated', payload => {
    setAccumulatorSecs(payload.speech_secs);
    setAccumulatorTrigger(payload.trigger_secs);
  });

  useTauriEvent<FastAccumulatorUpdatedEvent>('fast_accumulator_updated', payload => {
    setFastAccumulatorSecs(payload.speech_secs);
    setFastAccumulatorTrigger(payload.trigger_secs);
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
        ? { ...entry, response_ms: payload.response_ms, segment_count: payload.segment_count, best_score: payload.best_score }
        : entry
    ));
  });

  useTauriEvent<SpeakerRenamedEvent>('speaker_renamed', payload => {
    setSegments(prev =>
      prev.map(s =>
        s.speaker_id === payload.speech_swift_id
          ? { ...s, display_name: payload.display_name }
          : s
      )
    );
  });

  useTauriEvent<SpeakersMergedEvent>('speakers_merged', payload => {
    setSegments(prev =>
      prev.map(s =>
        s.speaker_id === payload.src_id
          ? { ...s, speaker_id: payload.dst_id, display_name: payload.dst_display_name }
          : s
      )
    );
  });

  useTauriEvent<SpeakerDeletedEvent>('speaker_deleted', payload => {
    setSegments(prev =>
      prev.map(s =>
        s.speaker_id === payload.speech_swift_id
          ? { ...s, speaker_id: null, display_name: null }
          : s
      )
    );
  });

  useTauriEvent<void>('speaker_registry_reset', () => {
    setSegments(prev =>
      prev.map(s => ({ ...s, speaker_id: null, display_name: null }))
    );
  });

  async function handleStart() {
    const sessionId = await startSession.mutateAsync(language);
    setSessionState({ status: 'recording', sessionId, startedAt: new Date() });
    setSegments([]);
    setShowNewSpeakerBanner(false);
    setElapsed(0);
    setPipelineEntries([]);
    setSegmentKinds({});
    setReplacedBySlowMap({});
    latestSegmentsRef.current = [];
    setAccumulatorSecs(0);
    setAccumulatorTrigger(10);
    setFastAccumulatorSecs(0);
    setFastAccumulatorTrigger(2);
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
    segmentKinds,
    replacedBySlowMap,
    elapsed,
    pipelineEntries,
    accumulatorSecs,
    accumulatorTrigger,
    fastAccumulatorSecs,
    fastAccumulatorTrigger,
    showNewSpeakerBanner,
    setShowNewSpeakerBanner,
    vadActive,
    micActive,
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
