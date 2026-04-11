import { Alert, Button } from 'flowbite-react';

interface Props {
  message: string;
  onRetry?: () => void;
}

export function QueryError({ message, onRetry }: Props) {
  return (
    <Alert color="failure" className="m-4">
      <div className="flex flex-col gap-2">
        <span className="font-medium">Failed to load data</span>
        <span className="font-mono text-sm break-all">{message}</span>
        {onRetry && (
          <div>
            <Button size="xs" color="failure" onClick={onRetry}>
              Retry
            </Button>
          </div>
        )}
      </div>
    </Alert>
  );
}
