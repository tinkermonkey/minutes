import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient, useMutation } from '@tanstack/react-query';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useSpeechSwiftStatus } from '../hooks/useSpeechSwiftStatus';
import { useStartSession, useStopSession } from '../hooks/useSession';
import { RecordButton } from '../components/RecordButton';
import { SessionStatusBadge } from '../components/SessionStatusBadge';
import { TranscriptPanel } from '../components/TranscriptPanel';
import { NewSpeakerBanner } from '../components/NewSpeakerBanner';
import { SpeechSwiftErrorPanel } from '../components/SpeechSwiftErrorPanel';
import type { Segment, SpeakerNotification } from '../types/transcript';

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
    return () => {
      appWindow.setTitle('Minutes');
    };
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

  // segment_added listener
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    listen<Segment>('segment_added', e => {
      setSegments(prev => [...prev, e.payload]);
    }).then(fn => { unlistenFn = fn; });
    return () => { unlistenFn?.(); };
  }, []);

  // new_speaker listener
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    listen<SpeakerNotification>('new_speaker', () => {
      setShowNewSpeakerBanner(true);
    }).then(fn => { unlistenFn = fn; });
    return () => { unlistenFn?.(); };
  }, []);

  // speech_swift_unreachable listener — update shared query cache
  // Note: __root.tsx also subscribes; both are safe (idempotent setQueryData)
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    listen('speech_swift_unreachable', () => {
      queryClient.setQueryData(['speech_swift_status'], false);
    }).then(fn => { unlistenFn = fn; });
    return () => { unlistenFn?.(); };
  }, [queryClient]);

  async function handleStart() {
    const sessionId = await startSession.mutateAsync();
    setSessionState({ status: 'recording', sessionId, startedAt: new Date() });
    setSegments([]);
    setShowNewSpeakerBanner(false);
    setElapsed(0);
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
      </div>

      {/* New speaker banner */}
      {showNewSpeakerBanner && (
        <NewSpeakerBanner onDismiss={() => setShowNewSpeakerBanner(false)} />
      )}

      {/* Transcript panel */}
      <TranscriptPanel
        segments={segments}
        isRecording={sessionState.status === 'recording'}
      />
    </div>
  );
}
