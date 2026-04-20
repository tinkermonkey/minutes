import type { Speaker } from '../../types/speaker';

interface Props {
  speakers:  Speaker[];
  isPending: boolean;
  progress:  { done: number; total: number } | null;
  error:     string | null;
  onConfirm: () => void;
  onCancel:  () => void;
}

const DOT_COLORS = [
  'bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-yellow-400',
  'bg-pink-500', 'bg-indigo-500', 'bg-orange-500', 'bg-teal-500',
];
function dotColor(id: number) { return DOT_COLORS[id % DOT_COLORS.length]; }

export function BulkDeleteModal({ speakers, isPending, progress, error, onConfirm, onCancel }: Props) {
  const n = speakers.length;
  const label = `${n} speaker${n !== 1 ? 's' : ''}`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
        <div className="p-6">
          <h2 className="text-base font-semibold text-gray-900 mb-1">Delete {label}?</h2>
          <p className="text-sm text-gray-500 mb-4">This cannot be undone.</p>

          <div className="max-h-48 overflow-y-auto border border-gray-200 rounded-lg divide-y divide-gray-100 mb-4">
            {speakers.map(s => {
              const name = s.display_name ?? `Unknown Speaker #${s.speech_swift_id}`;
              const isUnrecognized = s.display_name === null;
              return (
                <div key={s.speech_swift_id} className="flex items-center gap-2 px-3 py-2">
                  <div className={`w-2 h-2 rounded-full flex-shrink-0 ${isUnrecognized ? 'bg-gray-400' : dotColor(s.speech_swift_id)}`} />
                  <span className={`text-sm flex-1 truncate ${isUnrecognized ? 'italic text-gray-400' : 'text-gray-900'}`}>
                    {name}
                  </span>
                  {isUnrecognized && (
                    <span className="text-xs px-1.5 py-0.5 rounded bg-amber-100 text-amber-700 flex-shrink-0">Unrecognized</span>
                  )}
                </div>
              );
            })}
          </div>

          {progress && (
            <div className="mb-3">
              <div className="flex justify-between text-xs text-gray-500 mb-1">
                <span>Deleting…</span>
                <span>{progress.done} / {progress.total}</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-1.5">
                <div
                  className="bg-red-500 h-1.5 rounded-full transition-all"
                  style={{ width: `${(progress.done / progress.total) * 100}%` }}
                />
              </div>
            </div>
          )}

          {error && (
            <p className="text-xs text-red-600 bg-red-50 border border-red-200 rounded p-2 mb-3">{error}</p>
          )}

          <div className="flex gap-2 justify-end">
            <button
              onClick={onCancel}
              disabled={isPending}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onClick={onConfirm}
              disabled={isPending}
              className="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg disabled:opacity-50"
            >
              {isPending ? 'Deleting…' : `Delete ${label}`}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
