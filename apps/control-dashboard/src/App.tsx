import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { UserProvider } from './auth';
import { HealthProvider } from './contexts/HealthContext';
import DashboardLayout from './layouts/DashboardLayout';
import MachinesPage from './pages/MachinesPage';
import SessionsPage from './pages/SessionsPage';
import MonitoringPage from './pages/MonitoringPage';
import HistoryPage from './pages/HistoryPage';
import PoliciesPage from './pages/PoliciesPage';
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
            <Route path="sessions" element={<SessionsPage />} />
            <Route path="monitoring" element={<MonitoringPage />} />
            <Route path="history" element={<HistoryPage />} />
            <Route path="policies" element={<PoliciesPage />} />
            <Route path="*" element={<NotFoundPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
      </HealthProvider>
    </UserProvider>
  );
}

export default App;
