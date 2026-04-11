interface Props {
  score: number; // 0.0–1.0
}

export function RelevanceBar({ score }: Props) {
  const pct = Math.round(score * 100);
  const colorClass =
    score >= 0.8
      ? 'bg-green-400'
      : score >= 0.6
        ? 'bg-yellow-400'
        : 'bg-gray-300';

  return (
    <div className="flex items-center gap-2">
      <div className="w-20 h-1 bg-gray-100 rounded-full overflow-hidden flex-shrink-0">
        <div
          className={`h-full rounded-full ${colorClass}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-xs text-gray-400">{pct}%</span>
    </div>
  );
}
