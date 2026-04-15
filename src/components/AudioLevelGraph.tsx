import { useRef, useEffect } from 'react';
import { useTauriEvent } from '../hooks/useTauriEvent';

interface Props {
  active: boolean;
  /** When true, newly recorded samples are stamped as VAD-active and tinted. */
  vadActive?: boolean;
}

interface Sample {
  level: number;
  /** Whether VAD was active at the moment this sample was captured. */
  vad: boolean;
}

const WINDOW_MS  = 10_000;
const TICK_MS    = 50;
const MAX_SAMPLES = WINDOW_MS / TICK_MS; // 200

// Grid lines at 2 s, 4 s, 6 s, 8 s back from the right edge — expressed as
// sample counts from the right (newest) end of the buffer.
const GRID_INTERVALS_S = [2, 4, 6, 8] as const;
const GRID_SAMPLES     = GRID_INTERVALS_S.map(s => (s * 1_000) / TICK_MS);

const VAD_TINT_COLOR = 'rgba(59, 130, 246, 0.15)'; // blue-500 at 15% opacity
const BG_COLOR       = '#f9fafb';
const GRID_COLOR     = '#e5e7eb';
const LABEL_COLOR    = '#9ca3af';
const LABEL_FONT     = '9px system-ui, sans-serif';
const LABEL_PADDING  = 2; // px from bottom

export function AudioLevelGraph({ active, vadActive = false }: Props) {
  const wrapperRef  = useRef<HTMLDivElement>(null);
  const canvasRef   = useRef<HTMLCanvasElement>(null);
  const samplesRef  = useRef<Sample[]>([]);
  // Mirror vadActive into a ref so the event handler always reads the latest
  // value without needing to be recreated on each render.
  const vadActiveRef = useRef(vadActive);
  vadActiveRef.current = vadActive;

  // Stable draw function stored in a ref so the ResizeObserver callback and
  // the Tauri event handler always call the latest version without re-creating
  // closures on every render.
  const drawRef = useRef<() => void>(() => undefined);

  drawRef.current = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    // CSS dimensions (logical pixels)
    const W = canvas.clientWidth;
    const H = canvas.clientHeight;
    if (W === 0 || H === 0) return;

    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    // Background
    ctx.fillStyle = BG_COLOR;
    ctx.fillRect(0, 0, W, H);

    const samples  = samplesRef.current;
    const count    = samples.length;
    const slotW    = W / MAX_SAMPLES;

    // x-offset so newest sample always lands at the right edge.
    // When the buffer is full, xStart = 0 (samples fill the entire canvas).
    // When the buffer is partially filled, xStart > 0 (left portion is blank).
    const xStart = (MAX_SAMPLES - count) * slotW;

    // Grid lines + labels
    // Reserve a small strip at the bottom for labels.
    const labelH = 11; // px for label strip
    const waveH  = H - labelH;

    ctx.strokeStyle = GRID_COLOR;
    ctx.lineWidth   = 1;
    ctx.fillStyle   = LABEL_COLOR;
    ctx.font        = LABEL_FONT;
    ctx.textAlign   = 'center';
    ctx.textBaseline = 'bottom';

    for (const samplesBack of GRID_SAMPLES) {
      // x position counting back from the right edge
      const x = W - samplesBack * slotW;
      if (x < 0) continue; // not enough data to show this mark yet

      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, waveH);
      ctx.stroke();

      const label = `-${(samplesBack * TICK_MS) / 1_000}s`;
      ctx.fillText(label, x, H - LABEL_PADDING);
    }

    // "now" label at the right edge
    ctx.textAlign = 'right';
    ctx.fillText('now', W - 2, H - LABEL_PADDING);

    // VAD tint — draw a subtle blue rectangle for each VAD-active sample slot,
    // spanning the full waveH so the tint is visible behind the waveform.
    if (count > 0) {
      ctx.fillStyle = VAD_TINT_COLOR;
      for (let i = 0; i < count; i++) {
        if (!samples[i].vad) continue;
        const x0 = xStart + i * slotW;
        ctx.fillRect(x0, 0, slotW, waveH);
      }
    }

    // Waveform fill — only draw if we have samples
    if (count > 0) {
      const grad = ctx.createLinearGradient(0, 0, 0, waveH);
      grad.addColorStop(0, 'rgba(59,130,246,0.85)');   // blue-500
      grad.addColorStop(1, 'rgba(191,219,254,0.5)');   // blue-200

      ctx.beginPath();
      ctx.moveTo(xStart, waveH);

      for (let i = 0; i < count; i++) {
        const x0 = xStart + i * slotW;
        const x1 = xStart + (i + 1) * slotW;
        const y  = waveH * (1 - samples[i].level);
        ctx.lineTo(x0, y);
        ctx.lineTo(x1, y);
      }

      ctx.lineTo(xStart + count * slotW, waveH);
      ctx.closePath();

      ctx.fillStyle = grad;
      ctx.fill();
    }

    ctx.restore();
  };

  // ResizeObserver — keeps canvas physical pixels in sync with layout size.
  useEffect(() => {
    const wrapper = wrapperRef.current;
    const canvas  = canvasRef.current;
    if (!wrapper || !canvas) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;

      const { width, height } = entry.contentRect;
      const dpr = window.devicePixelRatio || 1;

      canvas.width  = Math.round(width  * dpr);
      canvas.height = Math.round(height * dpr);
      canvas.style.width  = `${width}px`;
      canvas.style.height = `${height}px`;

      drawRef.current();
    });

    observer.observe(wrapper);
    return () => observer.disconnect();
  }, []);

  // When active transitions false, clear the buffer and redraw.
  const prevActiveRef = useRef(active);
  useEffect(() => {
    if (prevActiveRef.current && !active) {
      samplesRef.current = [];
      drawRef.current();
    }
    prevActiveRef.current = active;
  }, [active]);

  // Subscribe to audio_level events; push into buffer and redraw — no setState.
  useTauriEvent<number>('audio_level', (payload) => {
    if (!active) return;

    // Apply sqrt scaling so the waveform spans the full graph height at normal
    // conversation volume (raw RMS values are typically 0.01–0.3).
    const level = Math.min(1, Math.max(0, Math.sqrt(payload) * 2.5));
    const samples = samplesRef.current;
    samples.push({ level, vad: vadActiveRef.current });

    // Evict oldest sample once the rolling window is full.
    if (samples.length > MAX_SAMPLES) {
      samples.shift();
    }

    drawRef.current();
  });

  return (
    <div
      ref={wrapperRef}
      className="w-full h-16 rounded-lg overflow-hidden border border-gray-200"
    >
      <canvas ref={canvasRef} className="block" />
    </div>
  );
}
