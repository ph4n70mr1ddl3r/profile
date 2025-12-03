import { Session } from 'next-auth';

export type SessionShape = {
  tenantId: string;
  userId: string;
  role: string;
};

export function extractSession(session: Session | null): SessionShape {
  if (!session?.user?.tenantId || !session.user.id || !session.user.role) {
    throw new Error('unauthorized');
  }

  return {
    tenantId: session.user.tenantId,
    userId: session.user.id,
    role: session.user.role
  };
}
