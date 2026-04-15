import { useState, useRef, useEffect } from 'react';
import { useRenameSpeaker } from '../../hooks/useSpeakers';
import type { Speaker } from '../../types/speaker';

interface Props {
  speaker:    Speaker;
  onRenamed?: () => void;
}

export function SpeakerNameField({ speaker }: Props) {
  const [editing, setEditing] = useState(false);
  const [value, setValue] = useState(speaker.display_name ?? '');
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const rename = useRenameSpeaker();

  useEffect(() => {
    if (editing) inputRef.current?.focus();
  }, [editing]);

  // Reset value when speaker prop changes (after successful rename)
  useEffect(() => {
    setValue(speaker.display_name ?? '');
  }, [speaker.display_name]);

  function startEdit() {
    setValue(speaker.display_name ?? '');
    setEditing(true);
  }

  function cancel() {
    setEditing(false);
    setError(null);
    setValue(speaker.display_name ?? '');
  }

  async function confirm() {
    const trimmed = value.trim();
    if (!trimmed) return;
    setError(null);
    try {
      await rename.mutateAsync({ speechSwiftId: speaker.speech_swift_id, name: trimmed });
      setEditing(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter') confirm();
    if (e.key === 'Escape') cancel();
  }

  if (editing) {
    return (
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <input
            ref={inputRef}
            type="text"
            value={value}
            onChange={e => setValue(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={rename.isPending}
            className={`border rounded px-2 py-1 text-sm focus:outline-none focus:ring-2 flex-1 ${error ? 'border-red-400 focus:ring-red-500' : 'border-blue-400 focus:ring-blue-500'}`}
          />
          <button
            onClick={confirm}
            disabled={!value.trim() || rename.isPending}
            className="px-2 py-1 text-sm bg-blue-600 text-white rounded disabled:opacity-50 hover:bg-blue-700"
          >
            {rename.isPending ? '...' : '✓'}
          </button>
          <button
            onClick={cancel}
            disabled={rename.isPending}
            className="px-2 py-1 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
          >
            ✕
          </button>
        </div>
        {error && <p className="text-xs text-red-600">{error}</p>}
      </div>
    );
  }

  const displayLabel = speaker.display_name ?? `Speaker ${speaker.speech_swift_id}`;

  return (
    <div className="flex items-center gap-2 group">
      <span className={`text-sm font-medium ${speaker.display_name ? 'text-gray-900' : 'text-gray-400'}`}>
        {displayLabel}
      </span>
      <button
        onClick={startEdit}
        className="text-xs text-gray-400 hover:text-gray-600 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        Edit
      </button>
    </div>
  );
}
