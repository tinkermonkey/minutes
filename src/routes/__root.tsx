import { useState } from 'react';
import { Outlet, Link, useRouterState } from '@tanstack/react-router';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTauriEvent } from '../hooks/useTauriEvent';
import { RouteErrorBoundary } from '../components/RouteErrorBoundary';
import { SettingsDrawer } from '../components/SettingsDrawer';
import { RecordingProvider, useRecording } from '../contexts/RecordingContext';
import { RecordButton } from '../components/RecordButton';
import { SessionStatusBadge } from '../components/SessionStatusBadge';
import { AudioMeter } from '../components/AudioMeter';
import { AccumulatorBar } from '../components/PipelineEventLog';

function RootLayoutInner() {
  const queryClient  = useQueryClient();
  const routerState  = useRouterState();
  const currentPath  = routerState.location.pathname;
  const [settingsOpen, setSettingsOpen] = useState(false);

  const { data: speechSwiftOk } = useQuery({
    queryKey: ['speech_swift_status'],
    queryFn: (): Promise<boolean> => invoke('get_speech_swift_status'),
    staleTime: Infinity,
  });

  useTauriEvent<void>('speech_swift_reachable', () => {
    queryClient.setQueryData(['speech_swift_status'], true);
  });

  const {
    sessionState,
    language,
    setLanguage,
    elapsed,
    accumulatorSecs,
    accumulatorTrigger,
    vadActive,
    handleStart,
    handleStop,
  } = useRecording();

  const isRecording = sessionState.status === 'recording';

  const navLinks = [
    { to: '/record',   label: 'Record'   },
    { to: '/speakers', label: 'Speakers' },
    { to: '/sessions', label: 'Sessions' },
    { to: '/search',   label: 'Search'   },
  ] as const;

  return (
    <div className="flex flex-col h-screen bg-gray-50">
      {/* Persistent top bar — recording controls always visible */}
      <header className="flex-shrink-0 h-14 bg-white border-b border-gray-200 flex items-center gap-4 px-4">
        <RecordButton
          status={sessionState.status}
          disabled={!speechSwiftOk}
          onStart={handleStart}
          onStop={handleStop}
        />
        <SessionStatusBadge status={sessionState.status} elapsedMs={elapsed} />
        <AudioMeter active={isRecording} vadActive={vadActive} />
        <AccumulatorBar secs={accumulatorSecs} trigger={accumulatorTrigger} />
        <select
          value={language}
          onChange={e => setLanguage(e.target.value as 'english' | 'auto')}
          disabled={isRecording}
          className="text-xs rounded border border-gray-200 bg-white px-2 py-1 text-gray-600 focus:outline-none focus:ring-1 focus:ring-blue-400 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <option value="english">English</option>
          <option value="auto">Auto</option>
        </select>
      </header>

      {/* Body */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <aside className="w-[220px] flex-shrink-0 bg-white border-r border-gray-200 flex flex-col">
          <div className="px-4 py-5 border-b border-gray-100">
            <h1 className="text-lg font-bold text-gray-900">Minutes</h1>
          </div>
          <nav className="flex-1 px-2 py-4 flex flex-col gap-1">
            {navLinks.map(({ to, label }) => {
              const isActive =
                currentPath === to || currentPath.startsWith(to + '/');
              return (
                <Link
                  key={to}
                  to={to}
                  className={`flex items-center px-3 py-2 text-sm rounded-lg transition-colors ${
                    isActive
                      ? 'bg-blue-50 text-blue-700 font-semibold'
                      : 'text-gray-700 hover:bg-gray-100'
                  }`}
                >
                  {label}
                </Link>
              );
            })}
          </nav>
          <div className="px-2 py-4 border-t border-gray-100">
            <button
              className="flex items-center px-3 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded-lg w-full transition-colors"
              onClick={() => setSettingsOpen(true)}
            >
              Settings
            </button>
          </div>
        </aside>

        {/* Main content */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Warning banner — shown on non-record routes; /record uses SpeechSwiftErrorPanel */}
          {speechSwiftOk === false && currentPath !== '/record' && (
            <div className="bg-amber-50 border-b border-amber-200 px-4 py-3 flex items-center gap-2 text-amber-800 text-sm">
              <span>Warning:</span>
              <span>speech-swift is unreachable — recording is disabled.</span>
            </div>
          )}
          <main className="flex-1 overflow-auto flex flex-col">
            <RouteErrorBoundary>
              <Outlet />
            </RouteErrorBoundary>
          </main>
        </div>
      </div>

      <SettingsDrawer isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

export function RootLayout() {
  return (
    <RecordingProvider>
      <RootLayoutInner />
    </RecordingProvider>
  );
}
