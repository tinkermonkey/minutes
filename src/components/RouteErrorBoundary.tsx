import { ErrorBoundary, type FallbackProps } from 'react-error-boundary';
import { Button } from 'flowbite-react';

function ErrorFallback({ error, resetErrorBoundary }: FallbackProps) {
  const message = error instanceof Error ? error.message : String(error);
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 p-12 text-center">
      <h2 className="text-xl font-semibold text-gray-900">Something went wrong</h2>
      <p className="font-mono text-sm text-red-600 bg-red-50 rounded-md px-4 py-2 max-w-lg break-all">
        {message}
      </p>
      <Button onClick={resetErrorBoundary}>Try again</Button>
    </div>
  );
}

interface Props {
  children: React.ReactNode;
}

export function RouteErrorBoundary({ children }: Props) {
  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>
      {children}
    </ErrorBoundary>
  );
}
