import { useState, useCallback } from 'react';
import { useTauriEvent } from './useTauriEvent';

/**
 * Tracks VAD (Voice Activity Detection) state from `vad_state` transition events.
 *
 * The Rust capture thread emits `vad_state` (boolean payload) on every
 * speech↔silence transition — directly from the webrtc-vad frame classifier,
 * not derived from chunk completion. This makes the indicator accurate in
 * real time rather than lagging by the 500 ms trailing-silence window.
 *
 * @param enabled - When false, vadActive is always false and transitions are
 *   ignored (prevents stale state after recording stops).
 */
export function useVadState(enabled: boolean): boolean {
  const [vadActive, setVadActive] = useState(false);

  const handleVadState = useCallback((active: boolean) => {
    if (!enabled) return;
    setVadActive(active);
  }, [enabled]);

  useTauriEvent<boolean>('vad_state', handleVadState);

  return vadActive;
}
