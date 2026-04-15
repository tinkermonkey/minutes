import { useSpeechSwiftStatus } from '../hooks/useSpeechSwiftStatus';
import { useRecording } from '../contexts/RecordingContext';
import { AudioLevelGraph } from '../components/AudioLevelGraph';
import { PipelineEventLog } from '../components/PipelineEventLog';
import { TranscriptPanel } from '../components/TranscriptPanel';
import { NewSpeakerBanner } from '../components/NewSpeakerBanner';
import { SpeechSwiftErrorPanel } from '../components/SpeechSwiftErrorPanel';
import { SessionSpeakersSidebar } from '../components/SessionSpeakersSidebar';

export function RecordRoute() {
  const { data: speechSwiftOk } = useSpeechSwiftStatus();
  const {
    retryHealth,
    sessionState,
    segments,
    pipelineEntries,
    showNewSpeakerBanner,
    setShowNewSpeakerBanner,
    vadActive,
  } = useRecording();

  const isRecording = sessionState.status === 'recording';

  if (speechSwiftOk === false) {
    return (
      <SpeechSwiftErrorPanel
        onRetry={() => retryHealth.mutate()}
        isRetrying={retryHealth.isPending}
      />
    );
  }

  return (
    <div className="flex h-full overflow-hidden">
      <div className="flex flex-col gap-4 p-6 flex-1 overflow-y-auto">
        {showNewSpeakerBanner && (
          <NewSpeakerBanner onDismiss={() => setShowNewSpeakerBanner(false)} />
        )}

        {sessionState.status !== 'idle' && (
          <AudioLevelGraph active={isRecording} vadActive={vadActive} />
        )}

        {(sessionState.status !== 'idle' || pipelineEntries.length > 0) && (
          <div className="flex flex-col gap-1">
            <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-wide px-1">
              Pipeline
            </h2>
            <PipelineEventLog entries={pipelineEntries} />
          </div>
        )}

        <TranscriptPanel segments={segments} isRecording={isRecording} />
      </div>

      <SessionSpeakersSidebar />
    </div>
  );
}
