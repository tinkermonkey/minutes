const widths = [80, 100, 60, 90, 75, 100, 55, 85, 70, 95, 65, 80];

export function SessionDetailSkeleton() {
  return (
    <div className="flex flex-col gap-6 p-6 animate-pulse">
      <div className="flex flex-col gap-3">
        <div className="h-6 w-64 bg-gray-200 rounded" />
        <div className="h-4 w-40 bg-gray-200 rounded" />
        <div className="flex gap-2">
          <div className="h-6 w-16 rounded-full bg-gray-200" />
          <div className="h-6 w-16 rounded-full bg-gray-200" />
          <div className="h-6 w-16 rounded-full bg-gray-200" />
        </div>
      </div>
      <div className="flex flex-col gap-3">
        {widths.map((w, i) => (
          <div key={i} className="h-4 bg-gray-200 rounded" style={{ width: `${w}%` }} />
        ))}
      </div>
    </div>
  );
}
