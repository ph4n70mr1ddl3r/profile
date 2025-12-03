import { z } from 'zod';

const envSchema = z
  .object({
    NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
    DATABASE_URL: z.string().url(),
    UPSTASH_REDIS_REST_URL: z.string().url(),
    UPSTASH_REDIS_REST_TOKEN: z.string().min(1),
    SENTRY_DSN: z.string().url(),
    NEXTAUTH_SECRET: z.string().min(1),
    NEXTAUTH_URL: z.string().url(),
    BASE_APP_URL: z.string().url()
  })
  .superRefine((val, ctx) => {
    if (!val.UPSTASH_REDIS_REST_URL || !val.UPSTASH_REDIS_REST_TOKEN) {
      ctx.addIssue({
        code: 'custom',
        message: 'Redis REST URL and token are required'
      });
    }
  });

type Env = z.infer<typeof envSchema>;

let cachedEnv: Env | null = null;

export function loadEnv(): Env {
  if (cachedEnv) {
    return cachedEnv;
  }
  const parsed = envSchema.safeParse(process.env);
  if (!parsed.success) {
    const issues = parsed.error.issues.map((issue) => `${issue.path.join('.')}: ${issue.message}`).join(', ');
    throw new Error(`Invalid environment configuration: ${issues}`);
  }
  cachedEnv = parsed.data;
  return cachedEnv;
}

export const env = loadEnv();
