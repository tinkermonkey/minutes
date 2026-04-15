import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Speaker } from '../types/speaker';

export const SPEAKERS_KEY = ['speakers'] as const;

export function useSpeakers() {
  return useQuery({
    queryKey: SPEAKERS_KEY,
    queryFn: (): Promise<Speaker[]> => invoke('get_speakers'),
  });
}

export function useRenameSpeaker() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ speechSwiftId, name }: { speechSwiftId: number; name: string }) =>
      invoke('rename_speaker', { speechSwiftId, name }),
    onSuccess: () => qc.invalidateQueries({ queryKey: SPEAKERS_KEY }),
  });
}

export function useMergeSpeakers() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ srcId, dstId }: { srcId: number; dstId: number }) =>
      invoke('merge_speakers', { srcId, dstId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: SPEAKERS_KEY }),
    onError: (e) => console.error('[merge_speakers]', e),
  });
}

export function useDeleteSpeaker() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (speechSwiftId: number) =>
      invoke('delete_speaker', { speechSwiftId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: SPEAKERS_KEY }),
  });
}

export function useResetRegistry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (): Promise<void> => invoke('reset_speaker_registry'),
    onSuccess: () => qc.invalidateQueries({ queryKey: SPEAKERS_KEY }),
  });
}

export function useSpeakerSamplePath(speechSwiftId: number) {
  return useQuery({
    queryKey: ['speaker_sample', speechSwiftId],
    queryFn: (): Promise<string | null> =>
      invoke('get_speaker_sample_path', { speechSwiftId }),
    enabled: speechSwiftId > 0,
  });
}
