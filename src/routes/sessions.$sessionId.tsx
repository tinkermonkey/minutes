import { useNavigate, useParams } from '@tanstack/react-router';
import { useSession, useSegments } from '../hooks/useSessions';
import { SessionDetailHeader } from '../components/sessions/SessionDetailHeader';
import { TranscriptReplayPanel } from '../components/sessions/TranscriptReplayPanel';
import { SessionDetailSkeleton } from '../components/sessions/SessionDetailSkeleton';
import { QueryError } from '../components/QueryError';
import { SessionSpeakersSidebar } from '../components/SessionSpeakersSidebar';

export function SessionDetailRoute() {
  const navigate = useNavigate();
  const { sessionId: sessionIdStr } = useParams({ strict: false }) as { sessionId: string };
  const sessionId = Number(sessionIdStr);

  const { data: session, isLoading: sessionLoading, isError: sessionError, error: sessionErr, refetch: refetchSession } = useSession(sessionId);
  const { data: segments = [], isLoading: segmentsLoading, isError: segmentsError, error: segmentsErr, refetch: refetchSegments } = useSegments(sessionId);

  if (sessionLoading || segmentsLoading) {
    return <SessionDetailSkeleton />;
  }

  if (sessionError) {
    return (
      <QueryError
        message={sessionErr instanceof Error ? sessionErr.message : String(sessionErr)}
        onRetry={() => refetchSession()}
      />
    );
  }

  if (segmentsError) {
    return (
      <QueryError
        message={segmentsErr instanceof Error ? segmentsErr.message : String(segmentsErr)}
        onRetry={() => refetchSegments()}
      />
    );
  }

  if (!session) {
    return <div className="p-6 text-gray-500">Session not found.</div>;
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="px-6 pt-6 pb-4 flex-shrink-0">
        <SessionDetailHeader
          session={session}
          onBack={() => navigate({ to: '/sessions' })}
        />
      </div>
      <div className="flex flex-1 min-h-0">
        <div className="flex-1 flex flex-col min-w-0 px-6 pb-6 overflow-hidden">
          <TranscriptReplayPanel segments={segments} />
        </div>
        <SessionSpeakersSidebar segments={segments} />
      </div>
    </div>
  );
}
