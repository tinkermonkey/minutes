interface Props {
  isPending: boolean;
  error:     string | null;
  onConfirm: () => void;
  onCancel:  () => void;
}

export function ResetRegistryModal({ isPending, error, onConfirm, onCancel }: Props) {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-40 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">Reset Speaker Registry</h2>
        <p className="text-sm text-gray-600 mb-2">
          This will permanently delete all speaker identities and voice data from the registry.
          All segments will be marked as unidentified.
        </p>
        <p className="text-sm text-red-600 mb-6">This cannot be undone.</p>
        {error && (
          <p className="text-sm text-red-600 bg-red-50 border border-red-200 rounded-lg px-3 py-2 mb-4">
            {error}
          </p>
        )}
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
            {isPending ? (
              <span className="flex items-center gap-2">
                <svg className="animate-spin h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
                </svg>
                Resetting...
              </span>
            ) : (
              'Reset Registry'
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
