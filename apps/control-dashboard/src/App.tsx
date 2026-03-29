import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { UserProvider } from './auth';
import { HealthProvider } from './contexts/HealthContext';
import DashboardLayout from './layouts/DashboardLayout';
import MachinesPage from './pages/MachinesPage';
import MachineDetailPage from './pages/MachineDetailPage';
import SessionsPage from './pages/SessionsPage';
import NewSessionPage from './pages/NewSessionPage';
import SessionDetailPage from './pages/SessionDetailPage';
import HistoryPage from './pages/HistoryPage';
import HistoryDetailPage from './pages/HistoryDetailPage';
import DocumentsPage from './pages/planning/DocumentsPage';
import DocumentDetailPage from './pages/planning/DocumentDetailPage';
import HierarchyPage from './pages/planning/HierarchyPage';
import NotificationsPage from './pages/NotificationsPage';
import NotFoundPage from './pages/NotFoundPage';

function GovernancePlaceholder() {
  return (
    <div className="flex items-center justify-center py-24">
      <div className="rounded-lg border border-secondary-200 bg-white px-8 py-12 text-center shadow-sm">
        <h2 className="text-2xl font-semibold text-secondary-900">Governance</h2>
        <p className="mt-2 text-secondary-500">Coming Soon</p>
      </div>
    </div>
  );
}

function App() {
  return (
    <UserProvider>
      <HealthProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<DashboardLayout />}>
            <Route index element={<Navigate to="/sessions" replace />} />
            <Route path="machines" element={<MachinesPage />} />
            <Route path="machines/:id" element={<MachineDetailPage />} />
            <Route path="sessions" element={<SessionsPage />} />
            <Route path="sessions/new" element={<NewSessionPage />} />
            <Route path="sessions/:id" element={<SessionDetailPage />} />
            <Route path="planning/documents" element={<DocumentsPage />} />
            <Route path="planning/documents/:shortCode" element={<DocumentDetailPage />} />
            <Route path="planning/hierarchy" element={<HierarchyPage />} />
            <Route path="planning/governance" element={<GovernancePlaceholder />} />
            <Route path="planning/quality" element={<Navigate to="/planning/governance" replace />} />
            <Route path="planning/rules" element={<Navigate to="/planning/governance" replace />} />
            <Route path="history" element={<HistoryPage />} />
            <Route path="history/:id" element={<HistoryDetailPage />} />
            <Route path="notifications" element={<NotificationsPage />} />
            <Route path="*" element={<NotFoundPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
      </HealthProvider>
    </UserProvider>
  );
}

export default App;
