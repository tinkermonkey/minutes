import { useNavigate, useParams } from '@tanstack/react-router';
import { useSession, useSegments } from '../hooks/useSessions';
import { SessionDetailHeader } from '../components/sessions/SessionDetailHeader';
import { TranscriptReplayPanel } from '../components/sessions/TranscriptReplayPanel';

export function SessionDetailRoute() {
  const navigate = useNavigate();
  const { sessionId: sessionIdStr } = useParams({ strict: false }) as { sessionId: string };
  const sessionId = Number(sessionIdStr);

  const { data: session, isLoading: sessionLoading } = useSession(sessionId);
  const { data: segments = [], isLoading: segmentsLoading } = useSegments(sessionId);

  if (sessionLoading || segmentsLoading) {
    return <div className="p-6 text-gray-400 text-sm">Loading session...</div>;
  }

  if (!session) {
    return <div className="p-6 text-gray-500">Session not found.</div>;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <SessionDetailHeader
        session={session}
        onBack={() => navigate({ to: '/sessions' })}
      />
      <TranscriptReplayPanel segments={segments} />
    </div>
  );
}
