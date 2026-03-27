import { NavLink } from 'react-router-dom';
import {
  ServerIcon,
  PlayIcon,
  ChartBarIcon,
  ClockIcon,
  ShieldCheckIcon,
} from '@heroicons/react/24/outline';

const navigation = [
  { name: 'Machines', href: '/machines', icon: ServerIcon },
  { name: 'Sessions', href: '/sessions', icon: PlayIcon },
  { name: 'Monitoring', href: '/monitoring', icon: ChartBarIcon },
  { name: 'History', href: '/history', icon: ClockIcon },
  { name: 'Policies', href: '/policies', icon: ShieldCheckIcon },
];

interface SidebarProps {
  onClose?: () => void;
}

export default function Sidebar({ onClose }: SidebarProps) {
  return (
    <nav className="flex flex-1 flex-col px-3 py-4">
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
                </>
              )}
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}
