import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function useStartSession() {
  return useMutation({
    mutationFn: (): Promise<number> => invoke('start_session'),
  });
}

export function useStopSession() {
  return useMutation({
    mutationFn: (sessionId: number): Promise<void> =>
      invoke('stop_session', { sessionId }),
  });
}
