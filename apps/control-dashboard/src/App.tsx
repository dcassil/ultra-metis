import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { UserProvider } from './auth';
import { HealthProvider } from './contexts/HealthContext';
import DashboardLayout from './layouts/DashboardLayout';
import MachinesPage from './pages/MachinesPage';
import MachineDetailPage from './pages/MachineDetailPage';
import SessionsPage from './pages/SessionsPage';
import NewSessionPage from './pages/NewSessionPage';
import SessionDetailPage from './pages/SessionDetailPage';
import MonitoringPage from './pages/MonitoringPage';
import HistoryPage from './pages/HistoryPage';
import PoliciesPage from './pages/PoliciesPage';
import ViolationsPage from './pages/ViolationsPage';
import NotFoundPage from './pages/NotFoundPage';

function App() {
  return (
    <UserProvider>
      <HealthProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<DashboardLayout />}>
            <Route index element={<Navigate to="/machines" replace />} />
            <Route path="machines" element={<MachinesPage />} />
            <Route path="machines/:id" element={<MachineDetailPage />} />
            <Route path="sessions" element={<SessionsPage />} />
            <Route path="sessions/new" element={<NewSessionPage />} />
            <Route path="sessions/:id" element={<SessionDetailPage />} />
            <Route path="monitoring" element={<MonitoringPage />} />
            <Route path="history" element={<HistoryPage />} />
            <Route path="policies" element={<PoliciesPage />} />
            <Route path="violations" element={<ViolationsPage />} />
            <Route path="*" element={<NotFoundPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
      </HealthProvider>
    </UserProvider>
  );
}

export default App;
