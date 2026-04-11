type SessionStatus = 'idle' | 'recording' | 'stopping';

interface Props {
  status:   SessionStatus;
  disabled: boolean;
  onStart:  () => void;
  onStop:   () => void;
}

export function RecordButton({ status, disabled, onStart, onStop }: Props) {
  const isDisabled = disabled || status === 'stopping';

  const handleClick = () => {
    if (status === 'idle') onStart();
    else if (status === 'recording') onStop();
  };

  let buttonClasses: string;
  let label: React.ReactNode;

  if (isDisabled) {
    buttonClasses = 'bg-gray-400 text-white cursor-not-allowed';
    if (status === 'stopping') {
      label = 'Stopping...';
    } else {
      label = 'Record';
    }
  } else if (status === 'recording') {
    buttonClasses = 'bg-red-600 hover:bg-red-700 text-white';
    label = (
      <>
        <span className="animate-pulse inline-block w-2 h-2 rounded-full bg-red-200 mr-2" />
        Stop
      </>
    );
  } else {
    buttonClasses = 'bg-green-600 hover:bg-green-700 text-white';
    label = 'Record';
  }

  return (
    <button
      onClick={handleClick}
      disabled={isDisabled}
      title={disabled ? 'speech-swift is not running — recording unavailable' : undefined}
      className={`inline-flex items-center px-4 py-2 rounded-lg text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 ${buttonClasses}`}
    >
      {label}
    </button>
  );
}
