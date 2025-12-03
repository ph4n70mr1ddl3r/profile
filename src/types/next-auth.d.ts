import NextAuth from 'next-auth';

declare module 'next-auth' {
  interface Session {
    user: {
      id: string;
      email?: string | null;
      tenantId: string;
      role: string;
    };
  }

  interface User {
    tenantId: string;
    role: string;
  }
}
