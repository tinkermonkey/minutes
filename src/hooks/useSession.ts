import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function useStartSession() {
  return useMutation({
    mutationFn: ({ language, label }: { language: string; label?: string }): Promise<number> =>
      invoke('start_session', { language, label }),
  });
}

export function useStopSession() {
  return useMutation({
    mutationFn: (sessionId: number): Promise<void> =>
      invoke('stop_session', { sessionId }),
  });
}

export function useRenameSession() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ sessionId, label }: { sessionId: number; label: string }): Promise<void> =>
      invoke('rename_session', { sessionId, label }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sessions'] });
    },
  });
}

export function useResumeSession() {
  return useMutation({
    mutationFn: ({ sessionId, language }: { sessionId: number; language?: string }): Promise<number> =>
      invoke('resume_session', { sessionId, language }),
  });
}
