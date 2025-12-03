import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { env } from '@/lib/config/env';
import { getRedisClient } from '@/lib/redis';
import { logger } from '@/lib/logger';
import { getRequestContext } from '@/lib/telemetry/request-context';

export async function GET(req: NextRequest) {
  const { requestId, tenantId } = getRequestContext(req);
  const redis = getRedisClient();

  let redisStatus: 'ok' | 'fallback' | 'error' = redis.enabled ? 'ok' : 'fallback';

  if (redis.enabled) {
    try {
      await redis.ping();
    } catch (error) {
      redisStatus = 'error';
      logger.error('Redis ping failed', { requestId, tenantId, error: (error as Error).message });
    }
  } else {
    logger.warn('Redis fallback active', { requestId, tenantId });
  }

  const data = {
    status: 'ok',
    env: env.NODE_ENV,
    timestamp: new Date().toISOString(),
    services: {
      redis: redisStatus,
      sentry: Boolean(env.SENTRY_DSN)
    }
  };

  const response = NextResponse.json(
    {
      data,
      meta: { requestId }
    },
    { status: 200 }
  );

  if (requestId) response.headers.set('x-request-id', requestId);
  if (tenantId) response.headers.set('x-tenant-id', tenantId);

  return response;
}
