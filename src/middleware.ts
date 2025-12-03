import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { randomUUID } from 'node:crypto';

export function middleware(request: NextRequest) {
  const headers = new Headers(request.headers);
  const requestId = headers.get('x-request-id') ?? randomUUID();
  headers.set('x-request-id', requestId);

  // Placeholder: downstream auth story will enforce tenant binding; default to passthrough for now.
  const tenantId = headers.get('x-tenant-id');
  if (tenantId) {
    headers.set('x-tenant-id', tenantId);
  }

  const response = NextResponse.next({
    request: {
      headers
    }
  });

  response.headers.set('x-request-id', requestId);
  if (tenantId) {
    response.headers.set('x-tenant-id', tenantId);
  }

  return response;
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico).*)']
};
