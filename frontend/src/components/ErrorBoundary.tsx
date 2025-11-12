import * as Sentry from '@sentry/react';
import { ReactNode } from 'react';
import { Button } from '@/components/ui/button';
import { AlertCircle, RefreshCw, Home } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

interface ErrorFallbackProps {
  error: Error;
  resetError: () => void;
}

function ErrorFallback({ error, resetError }: ErrorFallbackProps) {
  const navigate = useNavigate();

  const handleGoHome = () => {
    navigate('/');
    resetError();
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="max-w-md w-full space-y-6 text-center">
        <div className="flex justify-center">
          <div className="rounded-full bg-destructive/10 p-4">
            <AlertCircle className="h-12 w-12 text-destructive" />
          </div>
        </div>

        <div className="space-y-2">
          <h1 className="text-2xl font-semibold tracking-tight">
            Something went wrong
          </h1>
          <p className="text-sm text-muted-foreground">
            An unexpected error occurred. The error has been logged and we'll look into it.
          </p>
        </div>

        {import.meta.env.DEV && (
          <div className="p-4 bg-muted rounded-lg text-left">
            <p className="text-xs font-mono text-destructive break-all">
              {error.message}
            </p>
          </div>
        )}

        <div className="flex gap-3 justify-center">
          <Button
            onClick={resetError}
            variant="outline"
            className="gap-2"
          >
            <RefreshCw className="h-4 w-4" />
            Try Again
          </Button>
          <Button
            onClick={handleGoHome}
            className="gap-2"
          >
            <Home className="h-4 w-4" />
            Go Home
          </Button>
        </div>
      </div>
    </div>
  );
}

interface PageErrorBoundaryProps {
  children: ReactNode;
  fallback?: (error: Error, resetError: () => void) => ReactNode;
}

export function PageErrorBoundary({
  children,
  fallback
}: PageErrorBoundaryProps) {
  return (
    <Sentry.ErrorBoundary
      fallback={({ error, resetError }) => {
        const errorObj = error instanceof Error ? error : new Error(String(error));
        const element = fallback ? fallback(errorObj, resetError) : <ErrorFallback error={errorObj} resetError={resetError} />;
        return element as React.ReactElement;
      }}
      showDialog={false}
      onError={(error, errorInfo) => {
        console.error('[Page Error]', error, errorInfo);
      }}
    >
      {children}
    </Sentry.ErrorBoundary>
  );
}

interface ComponentErrorBoundaryProps {
  children: ReactNode;
  fallbackMessage?: string;
}

export function ComponentErrorBoundary({
  children,
  fallbackMessage = 'Failed to load this component'
}: ComponentErrorBoundaryProps) {
  return (
    <Sentry.ErrorBoundary
      fallback={({ error, resetError }) => {
        const errorObj = error instanceof Error ? error : new Error(String(error));
        return (
          <div className="p-4 border border-destructive/50 rounded-lg bg-destructive/5">
            <div className="flex items-start gap-3">
              <AlertCircle className="h-5 w-5 text-destructive flex-shrink-0 mt-0.5" />
              <div className="flex-1 space-y-2">
                <p className="text-sm font-medium text-destructive">
                  {fallbackMessage}
                </p>
                {import.meta.env.DEV && (
                  <p className="text-xs font-mono text-muted-foreground">
                    {errorObj.message}
                  </p>
                )}
                <Button
                  onClick={resetError}
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs"
                >
                  Retry
                </Button>
              </div>
            </div>
          </div>
        );
      }}
      showDialog={false}
    >
      {children}
    </Sentry.ErrorBoundary>
  );
}
