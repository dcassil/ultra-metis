import { Bars3Icon } from '@heroicons/react/24/outline';
import { useCurrentUser } from '../auth';
import ConnectionIndicator from './ConnectionIndicator';

interface HeaderProps {
  onMenuToggle: () => void;
}

export default function Header({ onMenuToggle }: HeaderProps) {
  const user = useCurrentUser();

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
          <span className="text-sm text-secondary-500">{user.name}</span>
        </div>
      </div>
    </header>
  );
}
