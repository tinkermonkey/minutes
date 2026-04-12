import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient, useMutation } from '@tanstack/react-query';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useSpeechSwiftStatus } from '../hooks/useSpeechSwiftStatus';
import { useStartSession, useStopSession } from '../hooks/useSession';
import { useTauriEvent } from '../hooks/useTauriEvent';
import { RecordButton } from '../components/RecordButton';
import { SessionStatusBadge } from '../components/SessionStatusBadge';
import { AudioMeter } from '../components/AudioMeter';
import { PipelineEventLog } from '../components/PipelineEventLog';
import { TranscriptPanel } from '../components/TranscriptPanel';
import { NewSpeakerBanner } from '../components/NewSpeakerBanner';
import { SpeechSwiftErrorPanel } from '../components/SpeechSwiftErrorPanel';
import type {
  Segment,
  SpeakerNotification,
  ChunkSentEvent,
  ChunkProcessedEvent,
  PipelineEntry,
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
    setSegments(prev => [...prev, payload]);
  });

  useTauriEvent<SpeakerNotification>('new_speaker', () => {
    setShowNewSpeakerBanner(true);
  });

  useTauriEvent<void>('speech_swift_unreachable', () => {
    queryClient.setQueryData(['speech_swift_status'], false);
  });

  useTauriEvent<ChunkSentEvent>('chunk_sent', payload => {
    setPipelineEntries(prev => [...prev, {
      start_ms:   payload.start_ms,
      end_ms:     payload.end_ms,
      sent_at_ms: payload.sent_at_ms,
    }]);
  });

  useTauriEvent<ChunkProcessedEvent>('chunk_processed', payload => {
    setPipelineEntries(prev => prev.map(entry =>
      entry.start_ms === payload.start_ms
        ? { ...entry,
            response_ms:   payload.response_ms,
            word_count:    payload.word_count,
            speaker_count: payload.speaker_count }
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
  }

  async function handleStop() {
    if (sessionState.status !== 'recording') return;
    const { sessionId } = sessionState;
    setSessionState({ status: 'stopping', sessionId });
    await stopSession.mutateAsync(sessionId);
    setSessionState({ status: 'idle' });
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
        <AudioMeter active={sessionState.status === 'recording'} />
      </div>

      {/* New speaker banner */}
      {showNewSpeakerBanner && (
        <NewSpeakerBanner onDismiss={() => setShowNewSpeakerBanner(false)} />
      )}

      {/* Pipeline event log — only shown during or after a recording */}
      {(sessionState.status !== 'idle' || pipelineEntries.length > 0) && (
        <div className="flex flex-col gap-1">
          <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
            Pipeline
          </h2>
          <PipelineEventLog entries={pipelineEntries} />
        </div>
      )}

      {/* Transcript panel */}
      <TranscriptPanel
        segments={segments}
        isRecording={sessionState.status === 'recording'}
      />
    </div>
  );
}
