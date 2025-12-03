import Link from 'next/link';

export default function Home() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-semibold">Creator App Bootstrap</h1>
      <p className="text-slate-700">
        Baseline Next.js, Prisma, Redis, and Sentry wiring with typed env validation and telemetry.
      </p>
      <ul className="list-disc space-y-2 pl-5 text-slate-700">
        <li>Env validation with Zod at <code>src/lib/config/env.ts</code></li>
        <li>Request ID middleware with structured logging and Sentry scrubbing</li>
        <li>Health endpoint at <code>/api/health</code> with envelope + requestId</li>
        <li>Prisma schema for tenants and users with migrations scaffolded</li>
      </ul>
      <Link href="/api/health" className="text-blue-600 underline">
        View health endpoint
      </Link>
    </div>
  );
}
