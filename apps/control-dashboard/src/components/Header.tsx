import { Link } from 'react-router-dom';
import { Bars3Icon, BellIcon } from '@heroicons/react/24/outline';
import { useCurrentUser } from '../auth';
import { useUnreadNotifications } from '../hooks/useUnreadNotifications';
import ConnectionIndicator from './ConnectionIndicator';

interface HeaderProps {
  onMenuToggle: () => void;
}

export default function Header({ onMenuToggle }: HeaderProps) {
  const user = useCurrentUser();
  const unreadCount = useUnreadNotifications();

  return (
    <header className="flex h-16 items-center border-b border-secondary-200 bg-white px-4 lg:px-6">
      <button
        type="button"
        className="-m-2.5 p-2.5 text-secondary-700 lg:hidden"
        onClick={onMenuToggle}
      >
        <span className="sr-only">Open sidebar</span>
        <Bars3Icon className="h-6 w-6" aria-hidden="true" />
      </button>

      <div className="flex flex-1 items-center gap-x-4 lg:gap-x-6">
        <h1 className="ml-3 text-lg font-semibold text-secondary-900 lg:ml-0">
          Cadre Control
        </h1>

        <div className="ml-auto flex items-center gap-x-4">
          <ConnectionIndicator />
          <Link
            to="/notifications"
            className="relative -m-1.5 p-1.5 text-secondary-500 hover:text-secondary-700 transition-colors"
            aria-label="Notifications"
          >
            <BellIcon className="h-6 w-6" aria-hidden="true" />
            {unreadCount > 0 && (
              <span className="absolute -top-0.5 -right-0.5 inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-danger-500 px-1 text-xs font-semibold text-white">
                {unreadCount > 99 ? '99+' : unreadCount}
              </span>
            )}
          </Link>
          <span className="text-sm text-secondary-500">{user.name}</span>
        </div>
      </div>
    </header>
  );
}
