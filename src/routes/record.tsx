import { useState } from 'react';
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
    segmentKinds,
    replacedBySlowMap,
    pipelineEntries,
    showNewSpeakerBanner,
    setShowNewSpeakerBanner,
    vadActive,
  } = useRecording();

  const [debugMode, setDebugMode] = useState(false);

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
  );
}
