import {
  createRouter,
  createRootRoute,
  createRoute,
  redirect,
} from '@tanstack/react-router';
import { RootLayout } from './routes/__root';
import { RecordRoute } from './routes/record';
import { SpeakersRoute } from './routes/speakers';
import { SessionsRoute } from './routes/sessions';
import { SearchRoute } from './routes/search';

const rootRoute = createRootRoute({
  component: RootLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/record' });
  },
});

const recordRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/record',
  component: RecordRoute,
});

const speakersRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/speakers',
  component: SpeakersRoute,
});

const sessionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/sessions',
  component: SessionsRoute,
});

const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/search',
  component: SearchRoute,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  recordRoute,
  speakersRoute,
  sessionsRoute,
  searchRoute,
]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
