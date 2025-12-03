import { Redis } from '@upstash/redis';
import { env } from './config/env';
import { logger } from './logger';

type RedisClient = {
  enabled: boolean;
  get: <T>(key: string) => Promise<T | null>;
  set: (key: string, value: unknown, options?: { ex?: number }) => Promise<string | null>;
  incrBy: (key: string, amount: number) => Promise<number>;
  ping: () => Promise<string>;
};

let cachedClient: RedisClient | null = null;

const fallbackClient: RedisClient = {
  enabled: false,
  async get() {
    return null;
  },
  async set() {
    return null;
  },
  async incrBy() {
    return 0;
  },
  async ping() {
    return 'fallback';
  }
};

export function getRedisClient(): RedisClient {
  if (cachedClient) return cachedClient;

  try {
    const redis = new Redis({
      url: env.UPSTASH_REDIS_REST_URL,
      token: env.UPSTASH_REDIS_REST_TOKEN
    });

    cachedClient = {
      enabled: true,
      get: (key) => redis.get(key),
      set: (key, value, options) => redis.set(key, value, { ex: options?.ex }),
      incrBy: (key, amount) => redis.incrby(key, amount),
      ping: () => redis.ping()
    };
  } catch (error) {
    logger.warn('Redis fallback active', { error: (error as Error).message });
    cachedClient = fallbackClient;
  }

  return cachedClient;
}
