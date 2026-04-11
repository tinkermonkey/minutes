export function SessionTableSkeleton() {
  return (
    <table className="w-full text-sm border-collapse">
      <tbody>
        {Array.from({ length: 8 }).map((_, i) => (
          <tr key={i} className="border-b border-gray-100 animate-pulse">
            <td className="p-3">
              <div className="w-32 h-4 bg-gray-200 rounded" />
            </td>
            <td className="p-3">
              <div className="w-16 h-4 bg-gray-200 rounded" />
            </td>
            <td className="p-3">
              <div className="flex gap-2">
                <div className="h-5 w-14 rounded-full bg-gray-200" />
                <div className="h-5 w-14 rounded-full bg-gray-200" />
              </div>
            </td>
            <td className="p-3">
              <div className="h-5 w-10 rounded-full bg-gray-200" />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
