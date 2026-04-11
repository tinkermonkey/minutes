import { Outlet, Link, useRouterState } from '@tanstack/react-router';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

export function RootLayout() {
  const queryClient = useQueryClient();
  const routerState = useRouterState();
  const currentPath = routerState.location.pathname;

  const { data: speechSwiftOk } = useQuery({
    queryKey: ['speech_swift_status'],
    queryFn: (): Promise<boolean> => invoke('get_speech_swift_status'),
    staleTime: Infinity,
  });

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    listen('speech_swift_unreachable', () => {
      queryClient.setQueryData(['speech_swift_status'], false);
    }).then(fn => { unlistenFn = fn; });
    return () => { unlistenFn?.(); };
  }, [queryClient]);

  const navLinks = [
    { to: '/record', label: 'Record' },
    { to: '/speakers', label: 'Speakers' },
    { to: '/sessions', label: 'Sessions' },
    { to: '/search', label: 'Search' },
  ] as const;

  return (
    <div className="flex h-screen bg-gray-50">
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
          <button className="flex items-center px-3 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded-lg w-full transition-colors">
            Settings
          </button>
        </div>
      </aside>

      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Warning banner */}
        {speechSwiftOk === false && (
          <div className="bg-amber-50 border-b border-amber-200 px-4 py-3 flex items-center gap-2 text-amber-800 text-sm">
            <span>Warning:</span>
            <span>speech-swift is unreachable — recording is disabled.</span>
          </div>
        )}
        <main className="flex-1 overflow-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
