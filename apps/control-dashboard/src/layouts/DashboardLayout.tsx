import { useState } from 'react';
import { Outlet } from 'react-router-dom';
import Header from '../components/Header';
import Sidebar from '../components/Sidebar';
import DisconnectedBanner from '../components/DisconnectedBanner';

export default function DashboardLayout() {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Mobile sidebar overlay */}
      {sidebarOpen && (
        <div className="relative z-50 lg:hidden">
          {/* Backdrop */}
          <div
            className="fixed inset-0 bg-secondary-900/80"
            onClick={() => setSidebarOpen(false)}
          />

          {/* Sidebar panel */}
          <div className="fixed inset-0 flex">
            <div className="relative mr-16 flex w-full max-w-64">
              <div className="flex grow flex-col overflow-y-auto bg-secondary-900">
                <div className="flex h-16 shrink-0 items-center px-6">
                  <span className="text-lg font-semibold text-white">
                    Cadre Control
                  </span>
                </div>
                <Sidebar onClose={() => setSidebarOpen(false)} />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Desktop sidebar */}
      <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-64 lg:flex-col">
        <div className="flex grow flex-col overflow-y-auto bg-secondary-900">
          <div className="flex h-16 shrink-0 items-center px-6">
            <span className="text-lg font-semibold text-white">
              Cadre Control
            </span>
          </div>
          <Sidebar />
        </div>
      </div>

      {/* Main content */}
      <div className="flex flex-1 flex-col lg:pl-64">
        <Header onMenuToggle={() => setSidebarOpen(true)} />
        <DisconnectedBanner />

        <main className="flex-1 overflow-y-auto bg-secondary-50 p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
