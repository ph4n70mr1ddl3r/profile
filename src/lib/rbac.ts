export type Role = 'brand_admin' | 'brand_member' | 'creator' | 'fan' | 'ops_admin';

export type SessionContext = {
  tenantId: string;
  userId: string;
  role: Role;
};

const roleHierarchy: Record<Role, number> = {
  brand_admin: 5,
  ops_admin: 4,
  brand_member: 3,
  creator: 2,
  fan: 1
};

export function assertRole(session: SessionContext, allowed: Role[]) {
  if (!allowed.includes(session.role)) {
    throw new Error('forbidden');
  }
}

export function requireTenant(session: SessionContext) {
  if (!session.tenantId) {
    throw new Error('tenant required');
  }
}

export function canAct(session: SessionContext, minimum: Role) {
  return roleHierarchy[session.role] >= roleHierarchy[minimum];
}
