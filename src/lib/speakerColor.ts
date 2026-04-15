const CHIP_COLORS = [
  'bg-blue-100 text-blue-800',
  'bg-green-100 text-green-800',
  'bg-purple-100 text-purple-800',
  'bg-yellow-100 text-yellow-800',
  'bg-pink-100 text-pink-800',
  'bg-indigo-100 text-indigo-800',
  'bg-orange-100 text-orange-800',
  'bg-teal-100 text-teal-800',
];

export function speakerColor(speakerId: number | null): string {
  if (speakerId === null) return 'bg-gray-100 text-gray-500';
  return CHIP_COLORS[speakerId % CHIP_COLORS.length];
}
