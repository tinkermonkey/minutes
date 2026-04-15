import { useState, useEffect } from 'react';
import { useTauriEvent } from '../hooks/useTauriEvent';

interface Props {
  active: boolean;
  /** When true, the VAD indicator segment lights up deep blue. */
  vadActive?: boolean;
}

// Number of discrete segments in the meter bar.
const SEGMENTS = 20;

// Segments above this index are yellow; above RED_THRESHOLD are red.
const YELLOW_THRESHOLD = 13;
const RED_THRESHOLD    = 17;

export function AudioMeter({ active, vadActive = false }: Props) {
  const [level, setLevel] = useState(0);

  useEffect(() => {
    if (!active) setLevel(0);
  }, [active]);

  useTauriEvent<number>('audio_level', payload => {
    if (!active) return;
    // RMS speech levels are typically 0.01–0.3; apply sqrt scaling so the
    // meter reads across its full range for normal conversation volume.
    setLevel(Math.min(1, Math.sqrt(payload) * 2.5));
  });

  const litSegments = Math.round(level * SEGMENTS);

  return (
    <div className="flex items-center gap-2" aria-label={`Microphone level: ${Math.round(level * 100)}%`}>
      <span className="text-xs text-gray-400 font-mono uppercase tracking-wide select-none">
        mic
      </span>
      <div className="flex gap-[2px]">
        {Array.from({ length: SEGMENTS }, (_, i) => {
          const lit = i < litSegments;
          let color: string;
          if (!lit) {
            color = 'bg-gray-200';
          } else if (i >= RED_THRESHOLD) {
            color = 'bg-red-500';
          } else if (i >= YELLOW_THRESHOLD) {
            color = 'bg-yellow-400';
          } else {
            color = 'bg-green-500';
          }
          return (
            <div
              key={i}
              className={`w-2 h-3 rounded-sm transition-colors duration-75 ${color}`}
            />
          );
        })}
      </div>
      {/* VAD indicator — one segment, same geometry as a meter segment */}
      <div
        className={`w-2 h-3 rounded-sm transition-colors duration-75 ${
          vadActive ? 'bg-blue-600' : 'bg-blue-200'
        }`}
        aria-label={vadActive ? 'Voice detected' : 'No voice detected'}
        title={vadActive ? 'VAD: active' : 'VAD: silent'}
      />
    </div>
  );
}
