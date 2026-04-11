interface Props {
  status:    'idle' | 'recording' | 'stopping';
  elapsedMs: number;
}

function formatElapsed(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

export function SessionStatusBadge({ status, elapsedMs }: Props) {
  if (status === 'idle') {
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
        Ready
      </span>
    );
  }

  if (status === 'stopping') {
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
        Stopping...
      </span>
    );
  }

  return (
    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-50 text-red-700">
      Recording · {formatElapsed(elapsedMs)}
    </span>
  );
}
