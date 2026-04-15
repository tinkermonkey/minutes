import { useMemo, useState } from 'react';
import { useRecording } from '../contexts/RecordingContext';
import { useRenameSpeaker } from '../hooks/useSpeakers';

interface SessionSpeaker {
  speakerId: number;
  speakerLabel: string;
  displayName: string | null;
}

const SPEAKER_COLORS = ['#2563EB', '#7C3AED', '#0891B2', '#059669', '#D97706', '#DC2626'];

function colorFor(idx: number): string {
  return SPEAKER_COLORS[idx % SPEAKER_COLORS.length];
}

interface SpeakerRowProps {
  speaker: SessionSpeaker;
  color: string;
  isEditing: boolean;
  draftName: string;
  isSaving: boolean;
  onStartEdit: (speaker: SessionSpeaker) => void;
  onDraftChange: (value: string) => void;
  onSave: () => void;
  onCancel: () => void;
}

function SpeakerRow({
  speaker,
  color,
  isEditing,
  draftName,
  isSaving,
  onStartEdit,
  onDraftChange,
  onSave,
  onCancel,
}: SpeakerRowProps) {
  if (isEditing) {
    return (
      <div className="px-3 py-2 rounded-lg bg-blue-50 border border-blue-200 mx-2">
        <div className="flex items-center gap-2 mb-2">
          <span
            className="w-2.5 h-2.5 rounded-full flex-shrink-0"
            style={{ backgroundColor: color }}
          />
          <span className="text-xs font-semibold text-gray-700">{speaker.speakerLabel}</span>
          <button
            onClick={onCancel}
            className="ml-auto text-gray-400 hover:text-gray-600 p-0.5"
            aria-label="Cancel editing"
          >
            ✕
          </button>
        </div>
        <input
          autoFocus
          value={draftName}
          onChange={e => onDraftChange(e.target.value)}
          onKeyDown={e => {
            if (e.key === 'Enter') onSave();
            if (e.key === 'Escape') onCancel();
          }}
          className="w-full text-xs rounded border border-blue-300 focus:border-blue-500 focus:outline-none px-2 py-1.5 mb-2 bg-white"
          placeholder="Enter name…"
        />
        <button
          onClick={onSave}
          disabled={!draftName.trim() || isSaving}
          className="w-full text-xs font-medium bg-blue-600 text-white rounded py-1.5 disabled:opacity-50 hover:bg-blue-700 transition-colors"
        >
          {isSaving ? 'Saving…' : 'Save name'}
        </button>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg group mx-2">
      <span
        className="w-2.5 h-2.5 rounded-full flex-shrink-0"
        style={{ backgroundColor: color }}
      />
      <div className="flex-1 min-w-0">
        <div className="text-xs font-semibold text-gray-700">{speaker.speakerLabel}</div>
        {speaker.displayName && (
          <div className="text-xs text-gray-400 truncate">{speaker.displayName}</div>
        )}
      </div>
      <button
        onClick={() => onStartEdit(speaker)}
        className="opacity-0 group-hover:opacity-100 p-1 rounded text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-opacity"
        aria-label={`Rename ${speaker.speakerLabel}`}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
        </svg>
      </button>
    </div>
  );
}

export function SessionSpeakersSidebar() {
  const { segments, updateSpeakerName } = useRecording();
  const rename = useRenameSpeaker();

  const [editingId, setEditingId] = useState<number | null>(null);
  const [draftName, setDraftName] = useState('');

  const speakers = useMemo<SessionSpeaker[]>(() => {
    const speakerMap = new Map<number, SessionSpeaker>();
    for (const seg of segments) {
      if (seg.speaker_id == null) continue;
      speakerMap.set(seg.speaker_id, {
        speakerId: seg.speaker_id,
        speakerLabel: seg.speaker_label ?? `Speaker ${seg.speaker_id}`,
        displayName: seg.display_name,
      });
    }
    return Array.from(speakerMap.values());
  }, [segments]);

  const recognized = useMemo(
    () => speakers.filter(s => s.displayName),
    [speakers],
  );
  const unrecognized = useMemo(
    () => speakers.filter(s => !s.displayName),
    [speakers],
  );

  function startEdit(speaker: SessionSpeaker) {
    setEditingId(speaker.speakerId);
    setDraftName(speaker.displayName ?? '');
  }

  function cancelEdit() {
    setEditingId(null);
    setDraftName('');
  }

  function handleSave() {
    if (!draftName.trim() || editingId == null) return;
    rename.mutate(
      { speechSwiftId: editingId, name: draftName.trim() },
      {
        onSuccess: () => {
          updateSpeakerName(editingId, draftName.trim());
          setEditingId(null);
          setDraftName('');
        },
      },
    );
  }

  return (
    <aside className="w-60 flex-shrink-0 flex flex-col bg-white border-l border-gray-200">
      <div className="h-12 flex items-center gap-2 px-4 border-b border-gray-100 flex-shrink-0">
        <span className="text-sm font-semibold text-gray-800">Session Speakers</span>
        <span className="ml-auto text-xs font-semibold text-gray-500 bg-gray-100 rounded-full px-2 py-0.5">
          {speakers.length}
        </span>
      </div>

      <div className="flex-1 overflow-y-auto py-3">
        {speakers.length === 0 && (
          <div className="px-4 text-xs text-gray-400 italic">No speakers yet</div>
        )}

        {recognized.length > 0 && (
          <div className="mb-2">
            <div className="px-3 mb-1 text-[10px] font-bold text-gray-400 tracking-wider uppercase">
              Recognized
            </div>
            {recognized.map(speaker => {
              const idx = speakers.indexOf(speaker);
              return (
                <SpeakerRow
                  key={speaker.speakerId}
                  speaker={speaker}
                  color={colorFor(idx)}
                  isEditing={editingId === speaker.speakerId}
                  draftName={draftName}
                  isSaving={rename.isPending}
                  onStartEdit={startEdit}
                  onDraftChange={setDraftName}
                  onSave={handleSave}
                  onCancel={cancelEdit}
                />
              );
            })}
          </div>
        )}

        {unrecognized.length > 0 && (
          <div>
            <div className="px-3 mb-1 text-[10px] font-bold text-gray-400 tracking-wider uppercase">
              Unrecognized
            </div>
            {unrecognized.map(speaker => {
              const idx = speakers.indexOf(speaker);
              return (
                <SpeakerRow
                  key={speaker.speakerId}
                  speaker={speaker}
                  color={colorFor(idx)}
                  isEditing={editingId === speaker.speakerId}
                  draftName={draftName}
                  isSaving={rename.isPending}
                  onStartEdit={startEdit}
                  onDraftChange={setDraftName}
                  onSave={handleSave}
                  onCancel={cancelEdit}
                />
              );
            })}
          </div>
        )}
      </div>

      <div className="px-4 py-3 border-t border-gray-100 text-xs text-gray-400">
        Names sync to Speakers registry
      </div>
    </aside>
  );
}
