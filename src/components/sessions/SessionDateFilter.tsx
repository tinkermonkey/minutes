interface Props {
  startDate: number | null;
  endDate:   number | null;
  onChange:  (start: number | null, end: number | null) => void;
}

export function SessionDateFilter({ startDate, endDate, onChange }: Props) {
  const startValue = startDate ? new Date(startDate).toISOString().split('T')[0] : '';
  const endValue = endDate ? new Date(endDate).toISOString().split('T')[0] : '';
  const today = new Date().toISOString().split('T')[0];

  return (
    <div className="flex items-center gap-2">
      <input
        type="date"
        value={startValue}
        max={today}
        onChange={e => onChange(e.target.value ? new Date(e.target.value).getTime() : null, endDate)}
        className="border border-gray-300 rounded px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <span className="text-gray-400 text-sm">to</span>
      <input
        type="date"
        value={endValue}
        min={startValue || undefined}
        max={today}
        onChange={e => onChange(startDate, e.target.value ? new Date(e.target.value).getTime() : null)}
        className="border border-gray-300 rounded px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      {(startDate || endDate) && (
        <button
          onClick={() => onChange(null, null)}
          className="text-sm text-gray-400 hover:text-gray-600"
        >
          Clear
        </button>
      )}
    </div>
  );
}
