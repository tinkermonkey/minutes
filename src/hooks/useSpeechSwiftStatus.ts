import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export function useSpeechSwiftStatus() {
  return useQuery({
    queryKey: ['speech_swift_status'],
    queryFn: (): Promise<boolean> => invoke('get_speech_swift_status'),
    staleTime: Infinity,
  });
}
