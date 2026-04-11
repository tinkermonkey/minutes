import type { SortBy, SortDir } from '../../types/session';

interface Props {
  label:   string;
  field:   SortBy;
  sortBy:  SortBy;
  sortDir: SortDir;
  onSort:  (field: SortBy) => void;
}

export function SortableHeader({ label, field, sortBy, sortDir, onSort }: Props) {
  const isActive = sortBy === field;

  return (
    <button
      onClick={() => onSort(field)}
      className="flex items-center gap-1 text-left font-medium text-gray-700 hover:text-gray-900"
    >
      {label}
      <span className="text-gray-400 text-xs">
        {isActive ? (sortDir === 'desc' ? '↓' : '↑') : '↕'}
      </span>
    </button>
  );
}
