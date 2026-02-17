import { Outlet } from '@tanstack/react-router';
import Sidebar from './Sidebar';

export default function Layout() {
  return (
    <div className="flex h-screen">
      {/* Animated background orbs */}
      <div className="fixed inset-0 -z-10 overflow-hidden">
        <div className="absolute -top-40 -left-40 w-96 h-96 rounded-full bg-indigo-600/20 blur-3xl animate-pulse" />
        <div className="absolute top-1/2 -right-20 w-80 h-80 rounded-full bg-purple-600/15 blur-3xl animate-pulse [animation-delay:2s]" />
        <div className="absolute -bottom-20 left-1/3 w-72 h-72 rounded-full bg-blue-600/15 blur-3xl animate-pulse [animation-delay:4s]" />
      </div>

      <Sidebar />
      <main className="flex-1 overflow-auto p-8">
        <Outlet />
      </main>
    </div>
  );
}
