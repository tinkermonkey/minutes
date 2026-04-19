import { useState, useCallback } from 'react';
import { useTauriEvent } from './useTauriEvent';

/**
 * Tracks whether the CPAL microphone capture is active.
 *
 * The Rust capture thread emits `mic_active` (boolean payload) when the CPAL
 * stream opens (true) and when it stops (false). This reflects the raw
 * hardware mic state, not VAD speech detection.
 */
export function useMicState(): boolean {
  const [micActive, setMicActive] = useState(false);
  const handle = useCallback((active: boolean) => setMicActive(active), []);
  useTauriEvent<boolean>('mic_active', handle);
  return micActive;
}
