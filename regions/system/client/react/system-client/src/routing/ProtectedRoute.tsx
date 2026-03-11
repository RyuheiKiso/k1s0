import type { ReactNode } from 'react';
import { useAuth } from '../auth/useAuth';

interface ProtectedRouteProps {
  children: ReactNode;
  fallback: ReactNode;
  requiredRoles?: string[];
}

export function ProtectedRoute({ children, fallback, requiredRoles }: ProtectedRouteProps) {
  const { isAuthenticated, user } = useAuth();

  const hasRequiredRole =
    requiredRoles == null ||
    requiredRoles.length === 0 ||
    requiredRoles.some((role) => user?.roles?.includes(role));

  return isAuthenticated && hasRequiredRole ? <>{children}</> : <>{fallback}</>;
}
