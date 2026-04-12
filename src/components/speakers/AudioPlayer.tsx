import { useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSpeakerSamplePath } from '../../hooks/useSpeakers';

interface Props {
  speechSwiftId: number;
}

function PlayIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" aria-hidden>
      <polygon points="2,1 11,6 2,11" />
    </svg>
  );
}

function StopIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" aria-hidden>
      <rect x="2" y="2" width="8" height="8" rx="1" />
    </svg>
  );
}

export function AudioPlayer({ speechSwiftId }: Props) {
  const { data: samplePath } = useSpeakerSamplePath(speechSwiftId);
  const audioRef  = useRef<HTMLAudioElement | null>(null);
  const blobUrlRef = useRef<string | null>(null);
  const [playing, setPlaying] = useState(false);
  const [error, setError]     = useState<string | null>(null);

  if (!samplePath) return null;

  async function handlePlay() {
    setError(null);
    try {
      // Read bytes via Tauri command — avoids asset protocol permission requirements.
      const bytes = await invoke<number[]>('read_audio_bytes', { path: samplePath });
      const blob  = new Blob([new Uint8Array(bytes)], { type: 'audio/wav' });
      const url   = URL.createObjectURL(blob);
      blobUrlRef.current = url;

      const audio = new Audio(url);
      audioRef.current = audio;
      audio.onended = () => {
        setPlaying(false);
        URL.revokeObjectURL(url);
        blobUrlRef.current = null;
      };
      await audio.play();
      setPlaying(true);
    } catch (e) {
      setError('Could not play sample');
      console.error('AudioPlayer error:', e);
    }
  }

  function handleStop() {
    audioRef.current?.pause();
    if (blobUrlRef.current) {
      URL.revokeObjectURL(blobUrlRef.current);
      blobUrlRef.current = null;
    }
    setPlaying(false);
  }

  return (
    <span className="inline-flex items-center gap-1">
      <button
        onClick={playing ? handleStop : handlePlay}
        className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded text-gray-700 border border-gray-200"
        title={playing ? 'Stop' : 'Play voice sample'}
      >
        {playing ? <StopIcon /> : <PlayIcon />}
        {playing ? 'Stop' : 'Play'}
      </button>
      {error && <span className="text-xs text-red-500">{error}</span>}
    </span>
  );
}
