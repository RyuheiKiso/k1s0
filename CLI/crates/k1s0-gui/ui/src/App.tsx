import { RouterProvider } from '@tanstack/react-router';
import { AuthProvider } from './lib/auth-provider';
import { WorkspaceProvider } from './lib/workspace-provider';
import { router } from './router';

export default function App() {
  return (
    <WorkspaceProvider>
      <AuthProvider>
        <RouterProvider router={router} />
      </AuthProvider>
    </WorkspaceProvider>
  );
}
