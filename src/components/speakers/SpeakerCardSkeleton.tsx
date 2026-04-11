function SkeletonCard() {
  return (
    <div className="bg-white border border-gray-200 rounded-xl p-4 flex flex-col gap-3 border-l-4 border-l-gray-200">
      <div className="h-5 w-40 bg-gray-200 rounded" />
      <div className="h-4 w-56 bg-gray-200 rounded" />
      <div className="h-4 w-32 bg-gray-200 rounded" />
      <div className="flex gap-2">
        <div className="h-8 w-20 rounded-lg bg-gray-200" />
        <div className="h-8 w-20 rounded-lg bg-gray-200" />
        <div className="h-8 w-20 rounded-lg bg-gray-200" />
      </div>
    </div>
  );
}

export function SpeakerCardSkeleton() {
  return (
    <div className="flex flex-col gap-3 animate-pulse">
      <SkeletonCard />
      <SkeletonCard />
      <SkeletonCard />
    </div>
  );
}
