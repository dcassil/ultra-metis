import { Switch } from '@headlessui/react'

interface ToggleProps {
  label: string
  description?: string
  checked: boolean
  onChange: (checked: boolean) => void
}

export function Toggle({ label, description, checked, onChange }: ToggleProps) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <span className="text-sm font-medium text-secondary-900">{label}</span>
        {description && <p className="text-sm text-secondary-500">{description}</p>}
      </div>
      <Switch
        checked={checked}
        onChange={onChange}
        className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 ${
          checked ? 'bg-primary-600' : 'bg-secondary-200'
        }`}
      >
        <span className="sr-only">{label}</span>
        <span
          className={`pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
            checked ? 'translate-x-5' : 'translate-x-0'
          }`}
        />
      </Switch>
    </div>
  )
}
