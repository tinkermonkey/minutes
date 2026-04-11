function SkeletonResultCard() {
  return (
    <div className="bg-white border border-gray-200 rounded-xl p-4 flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <div className="h-4 w-32 bg-gray-200 rounded" />
        <div className="h-4 w-16 bg-gray-200 rounded" />
      </div>
      <div className="h-5 w-full bg-gray-200 rounded" />
      <div className="h-4 w-3/4 bg-gray-200 rounded" />
      <div className="h-4 w-24 bg-gray-200 rounded" />
    </div>
  );
}

export function SearchResultSkeleton() {
  return (
    <div className="flex flex-col gap-3 animate-pulse">
      <SkeletonResultCard />
      <SkeletonResultCard />
      <SkeletonResultCard />
      <SkeletonResultCard />
      <SkeletonResultCard />
    </div>
  );
}
