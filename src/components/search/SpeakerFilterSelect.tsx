import { useSpeakers } from '../../hooks/useSpeakers';

interface Props {
  value:    number | null;
  onChange: (id: number | null) => void;
}

export function SpeakerFilterSelect({ value, onChange }: Props) {
  const { data: speakers = [], isLoading } = useSpeakers();

  // Named speakers alphabetically first, then unnamed by speech_swift_id
  const sorted = [...speakers].sort((a, b) => {
    if (a.display_name && b.display_name) return a.display_name.localeCompare(b.display_name);
    if (a.display_name && !b.display_name) return -1;
    if (!a.display_name && b.display_name) return 1;
    return a.speech_swift_id - b.speech_swift_id;
  });

  return (
    <select
      value={value ?? ''}
      onChange={e => onChange(e.target.value ? Number(e.target.value) : null)}
      disabled={isLoading}
      className="border border-gray-300 rounded px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
    >
      <option value="">All Speakers</option>
      {sorted.map(s => (
        <option key={s.speech_swift_id} value={s.speech_swift_id}>
          {s.display_name ?? `Unknown (Speaker ${s.speech_swift_id})`}
        </option>
      ))}
    </select>
  );
}
