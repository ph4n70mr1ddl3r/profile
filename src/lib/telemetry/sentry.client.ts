import * as Sentry from '@sentry/nextjs';
import { env } from '../config/env';

let initialized = false;

export function initSentryClient() {
  if (initialized) return;

  Sentry.init({
    dsn: env.SENTRY_DSN,
    tracesSampleRate: 0.2,
    sendDefaultPii: false,
    beforeSend(event) {
      if (event.request?.headers) {
        delete event.request.headers['cookie'];
        delete event.request.headers['authorization'];
      }
      return event;
    }
  });

  initialized = true;
}
