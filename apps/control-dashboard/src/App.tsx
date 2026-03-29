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
import GovernancePage from './pages/planning/GovernancePage';
import NotFoundPage from './pages/NotFoundPage';

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
            <Route path="planning/governance" element={<GovernancePage />} />
            <Route path="planning/quality" element={<Navigate to="/planning/governance?tab=quality" replace />} />
            <Route path="planning/rules" element={<Navigate to="/planning/governance?tab=rules" replace />} />
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
