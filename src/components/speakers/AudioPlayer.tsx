import { useRef, useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useSpeakerSamplePath } from '../../hooks/useSpeakers';

interface Props {
  speechSwiftId: number;
}

export function AudioPlayer({ speechSwiftId }: Props) {
  const { data: samplePath } = useSpeakerSamplePath(speechSwiftId);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [playing, setPlaying] = useState(false);

  if (!samplePath) return null;

  function handlePlay() {
    const url = convertFileSrc(samplePath!);
    audioRef.current = new Audio(url);
    audioRef.current.play();
    audioRef.current.onended = () => setPlaying(false);
    setPlaying(true);
  }

  function handleStop() {
    audioRef.current?.pause();
    setPlaying(false);
  }

  return (
    <button
      onClick={playing ? handleStop : handlePlay}
      className="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded text-gray-700 border border-gray-200"
      title={playing ? 'Stop' : 'Play voice sample'}
    >
      {playing ? 'Stop' : 'Listen'}
    </button>
  );
}
