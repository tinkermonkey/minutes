import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function useStartSession() {
  return useMutation({
    mutationFn: (language: string): Promise<number> =>
      invoke('start_session', { language }),
  });
}

export function useStopSession() {
  return useMutation({
    mutationFn: (sessionId: number): Promise<void> =>
      invoke('stop_session', { sessionId }),
  });
}
