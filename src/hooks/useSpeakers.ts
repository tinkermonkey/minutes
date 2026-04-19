import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Speaker, SimilarSpeaker, SpeakerDetail } from '../types/speaker';

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
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: SPEAKERS_KEY });
      qc.invalidateQueries({ queryKey: ['segments'] });
    },
  });
}

export function useMergeSpeakers() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ srcId, dstId }: { srcId: number; dstId: number }) =>
      invoke('merge_speakers', { srcId, dstId }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: SPEAKERS_KEY });
      qc.invalidateQueries({ queryKey: ['segments'] });
      qc.invalidateQueries({ queryKey: ['similar_speakers'] });
    },
    onError: (e) => console.error('[merge_speakers]', e),
  });
}

export function useDeleteSpeaker() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (speechSwiftId: number) =>
      invoke('delete_speaker', { speechSwiftId }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: SPEAKERS_KEY });
      qc.invalidateQueries({ queryKey: ['segments'] });
      qc.invalidateQueries({ queryKey: ['similar_speakers'] });
    },
  });
}

export function useResetRegistry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (): Promise<void> => invoke('reset_speaker_registry'),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: SPEAKERS_KEY });
      qc.invalidateQueries({ queryKey: ['segments'] });
    },
  });
}

export function useSimilarSpeakers(speechSwiftId: number | null) {
  return useQuery({
    queryKey: ['similar_speakers', speechSwiftId ?? 0],
    queryFn: (): Promise<SimilarSpeaker[]> =>
      invoke('get_similar_speakers', { speechSwiftId, limit: 10 }),
    enabled: speechSwiftId !== null,
  });
}

export function useSpeakerDetail(speechSwiftId: number | null) {
  return useQuery({
    queryKey: ['speaker_detail', speechSwiftId ?? 0],
    queryFn: (): Promise<SpeakerDetail> =>
      invoke('get_speaker_detail', { speechSwiftId }),
    enabled: speechSwiftId !== null,
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
