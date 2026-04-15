import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { SessionFilter, SessionsPage, Session, SegmentWithSpeaker } from '../types/session';

export function useSessions(filter: SessionFilter) {
  return useQuery({
    queryKey: ['sessions', filter],
    queryFn: (): Promise<SessionsPage> => invoke('get_sessions', { filter }),
    placeholderData: (prev) => prev,
  });
}

export function useSession(sessionId: number) {
  return useQuery({
    queryKey: ['session', sessionId],
    queryFn: (): Promise<Session | null> => invoke('get_session', { sessionId }),
    enabled: sessionId > 0,
  });
}

export function useSegments(sessionId: number) {
  return useQuery({
    queryKey: ['segments', sessionId],
    queryFn: (): Promise<SegmentWithSpeaker[]> => invoke('get_segments', { sessionId }),
    enabled: sessionId > 0,
  });
}

export function useDeleteAllSessions() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (): Promise<void> => invoke('delete_all_sessions'),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['sessions'] }),
  });
}
