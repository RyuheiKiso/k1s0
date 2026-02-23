import React, { type ReactNode } from 'react';
import { useAuth } from '../auth/useAuth';

interface ProtectedRouteProps {
  children: ReactNode;
  fallback: ReactNode;
}

export function ProtectedRoute({ children, fallback }: ProtectedRouteProps) {
  const { isAuthenticated } = useAuth();
  return isAuthenticated ? <>{children}</> : <>{fallback}</>;
}
