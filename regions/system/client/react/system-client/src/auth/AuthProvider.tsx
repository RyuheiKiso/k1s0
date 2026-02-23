import React, { useState, type ReactNode } from 'react';
import { AuthContext, type User } from './AuthContext';

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);

  const login = async ({ username }: { username: string; password: string }) => {
    // 実際の実装では API を呼び出す
    setUser({ id: `user-${Date.now()}`, username });
  };

  const logout = () => {
    setUser(null);
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated: user !== null,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
