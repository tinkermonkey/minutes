import type { Speaker } from '../../types/speaker';

interface Props {
  src:       Speaker;
  dst:       Speaker;
  onConfirm: () => void;
  onCancel:  () => void;
  isPending: boolean;
}

export function MergeConfirmModal({ src, dst, onConfirm, onCancel, isPending }: Props) {
  const srcName = src.display_name ?? `Speaker ${src.speech_swift_id}`;
  const dstName = dst.display_name ?? `Speaker ${dst.speech_swift_id}`;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-40 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Merge Speakers</h2>
        <p className="text-sm text-gray-600 mb-2">
          <strong>{srcName}</strong> will be merged into <strong>{dstName}</strong>.
        </p>
        <p className="text-sm text-gray-600 mb-6">
          All transcript segments attributed to {srcName} will be re-attributed to {dstName}.
          This cannot be undone.
        </p>
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
            className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 rounded-lg text-white disabled:opacity-50"
          >
            {isPending ? 'Merging...' : 'Merge'}
          </button>
        </div>
      </div>
    </div>
  );
}
