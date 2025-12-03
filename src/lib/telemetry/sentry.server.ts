import * as Sentry from '@sentry/nextjs';
import { env } from '../config/env';

let initialized = false;

export function initSentryServer() {
  if (initialized) return;

  Sentry.init({
    dsn: env.SENTRY_DSN,
    tracesSampleRate: 0.2,
    sendDefaultPii: false,
    beforeSend(event) {
      if (event.user) {
        delete event.user.ip_address;
        delete event.user.email;
      }
      if (event.request?.headers) {
        delete event.request.headers['cookie'];
        delete event.request.headers['authorization'];
      }
      return event;
    }
  });

  initialized = true;
}

export function withSentryScope<T>(fn: () => T, context?: { requestId?: string; tenantId?: string }) {
  initSentryServer();
  return Sentry.withScope((scope) => {
    if (context?.requestId) scope.setTag('request_id', context.requestId);
    if (context?.tenantId) scope.setTag('tenant_id', context.tenantId);
    return fn();
  });
}
