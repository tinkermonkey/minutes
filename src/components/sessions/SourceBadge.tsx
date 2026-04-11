interface Props {
  source: string;
}

export function SourceBadge({ source }: Props) {
  if (source === 'mic') {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-700">
        Mic
      </span>
    );
  }
  return (
    <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
      File
    </span>
  );
}
