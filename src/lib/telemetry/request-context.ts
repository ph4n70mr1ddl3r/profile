import type { NextRequest } from 'next/server';

export const getRequestContext = (req: NextRequest) => {
  const requestId = req.headers.get('x-request-id') ?? undefined;
  const tenantId = req.headers.get('x-tenant-id') ?? undefined;
  return { requestId, tenantId };
};
