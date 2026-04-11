import { Link } from '@tanstack/react-router';

interface Props {
  onDismiss: () => void;
}

export function NewSpeakerBanner({ onDismiss }: Props) {
  return (
    <div className="bg-amber-50 border border-amber-200 rounded-lg px-4 py-3 flex items-center justify-between text-amber-800 text-sm">
      <span>
        Unknown speaker detected — name them in the{' '}
        <Link to="/speakers" className="underline font-medium">
          Speaker Registry
        </Link>
      </span>
      <button
        onClick={onDismiss}
        className="ml-4 text-amber-600 hover:text-amber-800"
      >
        Dismiss
      </button>
    </div>
  );
}
