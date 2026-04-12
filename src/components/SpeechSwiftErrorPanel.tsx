import { Button, Spinner } from 'flowbite-react';

interface Props {
  onRetry: () => void;
  isRetrying: boolean;
}

export function SpeechSwiftErrorPanel({ onRetry, isRetrying }: Props) {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-6 p-12 text-center">
      <div className="w-16 h-16 rounded-full bg-red-100 flex items-center justify-center">
        <svg className="w-8 h-8 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
        </svg>
      </div>

      <div className="flex flex-col gap-2">
        <h2 className="text-xl font-semibold text-gray-900">speech-swift is not running</h2>
        <p className="text-gray-600 max-w-md">
          Minutes uses speech-swift to transcribe and identify speakers. It runs as a separate process on your Mac.
        </p>
      </div>

      <div className="flex flex-col items-center gap-3">
        <p className="text-gray-700 text-sm">To start it, open a terminal and run:</p>
        <code className="bg-gray-100 rounded-md px-4 py-2 text-sm font-mono text-left">
          ./speech-swift-audio-server
        </code>
        <p className="text-gray-500 text-xs">Listens on port 8080 by default.</p>
      </div>

      <Button disabled={isRetrying} onClick={onRetry}>
        {isRetrying && <Spinner size="sm" className="mr-2" />}
        {isRetrying ? 'Checking…' : 'Retry connection'}
      </Button>
    </div>
  );
}
