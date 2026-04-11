import type { Speaker } from '../../types/speaker';

interface Props {
  speaker:   Speaker;
  onConfirm: () => void;
  onCancel:  () => void;
  isPending: boolean;
}

export function DeleteConfirmModal({ speaker, onConfirm, onCancel, isPending }: Props) {
  const name = speaker.display_name ?? `Speaker ${speaker.speech_swift_id}`;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-40 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Delete Speaker</h2>
        <p className="text-sm text-gray-600 mb-2">
          Delete <strong>{name}</strong>? Their transcript segments will remain
          but will no longer be attributed to a named speaker.
        </p>
        <p className="text-sm text-red-600 mb-6">This cannot be undone.</p>
        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            disabled={isPending}
            className="px-4 py-2 text-sm bg-gray-100 hover:bg-gray-200 rounded-lg text-gray-700"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isPending}
            className="px-4 py-2 text-sm bg-red-600 hover:bg-red-700 rounded-lg text-white disabled:opacity-50"
          >
            {isPending ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      </div>
    </div>
  );
}
