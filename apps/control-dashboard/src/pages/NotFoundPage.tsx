import { Link } from 'react-router-dom';

export default function NotFoundPage() {
  return (
    <div className="flex items-center justify-center py-24">
      <div className="rounded-lg border border-secondary-200 bg-white px-4 py-12 sm:px-8 text-center shadow-sm">
        <h2 className="text-2xl font-semibold text-secondary-900">404</h2>
        <p className="mt-2 text-secondary-500">Page not found</p>
        <Link
          to="/machines"
          className="mt-4 inline-block rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white hover:bg-primary-700"
        >
          Go to Machines
        </Link>
      </div>
    </div>
  );
}
