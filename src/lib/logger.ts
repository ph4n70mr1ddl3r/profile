type LogLevel = 'info' | 'warn' | 'error';

type LogContext = {
  requestId?: string;
  tenantId?: string;
  [key: string]: unknown;
};

const base = (level: LogLevel, message: string, context: LogContext = {}) => {
  const payload = {
    level,
    message,
    requestId: context.requestId,
    tenantId: context.tenantId,
    timestamp: new Date().toISOString(),
    ...context
  };

  // eslint-disable-next-line no-console
  console[level](JSON.stringify(payload));
};

export const logger = {
  info: (message: string, context?: LogContext) => base('info', message, context),
  warn: (message: string, context?: LogContext) => base('warn', message, context),
  error: (message: string, context?: LogContext) => base('error', message, context)
};
