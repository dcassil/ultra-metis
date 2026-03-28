import { NavLink, Link } from 'react-router-dom';
import {
  ServerIcon,
  PlayIcon,
  ChartBarIcon,
  ClockIcon,
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  PlusIcon,
} from '@heroicons/react/24/outline';
import { usePendingCount } from '../hooks/usePendingCount';

const navigation = [
  { name: 'Machines', href: '/machines', icon: ServerIcon, showPendingBadge: true },
  { name: 'Sessions', href: '/sessions', icon: PlayIcon, showPendingBadge: false },
  { name: 'Monitoring', href: '/monitoring', icon: ChartBarIcon, showPendingBadge: false },
  { name: 'History', href: '/history', icon: ClockIcon, showPendingBadge: false },
  { name: 'Policies', href: '/policies', icon: ShieldCheckIcon, showPendingBadge: false },
  { name: 'Violations', href: '/violations', icon: ExclamationTriangleIcon, showPendingBadge: false },
];

interface SidebarProps {
  onClose?: () => void;
}

export default function Sidebar({ onClose }: SidebarProps) {
  const pendingCount = usePendingCount();

  return (
    <nav className="flex flex-1 flex-col px-3 py-4">
      <div className="mb-4">
        <Link
          to="/sessions/new"
          onClick={onClose}
          className="flex items-center justify-center gap-2 rounded-md bg-primary-600 px-3 py-2 text-sm font-medium text-white hover:bg-primary-700 transition-colors"
        >
          <PlusIcon className="h-4 w-4" aria-hidden="true" />
          Start Session
        </Link>
      </div>
      <ul className="flex flex-1 flex-col gap-y-1">
        {navigation.map((item) => (
          <li key={item.name}>
            <NavLink
              to={item.href}
              onClick={onClose}
              className={({ isActive }) =>
                [
                  'group flex items-center gap-x-3 rounded-md px-3 py-2 text-sm font-medium',
                  isActive
                    ? 'bg-primary-600 text-white'
                    : 'text-secondary-300 hover:bg-secondary-700 hover:text-white',
                ].join(' ')
              }
            >
              {({ isActive }) => (
                <>
                  <item.icon
                    className={[
                      'h-5 w-5 shrink-0',
                      isActive
                        ? 'text-white'
                        : 'text-secondary-400 group-hover:text-white',
                    ].join(' ')}
                    aria-hidden="true"
                  />
                  {item.name}
                  {item.showPendingBadge && pendingCount > 0 && (
                    <span className="ml-auto inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-warning-500 px-1.5 text-xs font-semibold text-white">
                      {pendingCount}
                    </span>
                  )}
                </>
              )}
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}
