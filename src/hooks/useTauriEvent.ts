import { useEffect, useRef } from 'react';
import { listen, type EventName } from '@tauri-apps/api/event';

/**
 * Subscribe to a Tauri event for the lifetime of the component.
 *
 * Handles the StrictMode double-invoke race: if React's cleanup fires before
 * the async `listen()` Promise resolves, a `cancelled` flag ensures the
 * listener is immediately unregistered when the Promise does resolve — so no
 * duplicate subscriptions leak.
 *
 * The handler is stored in a ref, so it always sees the latest closure values
 * without triggering a re-subscribe on every render.
 */
export function useTauriEvent<T>(
  event: EventName,
  handler: (payload: T) => void,
): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen<T>(event, e => handlerRef.current(e.payload)).then(fn => {
      if (cancelled) {
        fn(); // effect already cleaned up — unsubscribe immediately
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [event]); // only re-subscribe if the event name itself changes
}
