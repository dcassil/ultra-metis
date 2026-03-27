import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from '@headlessui/react'
import { CheckIcon, ChevronUpDownIcon } from '@heroicons/react/20/solid'

interface SelectOption {
  value: string
  label: string
}

interface SelectProps {
  label: string
  options: SelectOption[]
  value: string
  onChange: (value: string) => void
  error?: string
  placeholder?: string
}

export function Select({ label, options, value, onChange, error, placeholder }: SelectProps) {
  const selected = options.find((o) => o.value === value)
  return (
    <div>
      <Listbox value={value} onChange={onChange}>
        <label className="block text-sm font-medium text-secondary-700">{label}</label>
        <div className="relative mt-1">
          <ListboxButton
            className={`relative w-full cursor-default rounded-md border bg-white py-2 pl-3 pr-10 text-left text-sm shadow-sm focus:outline-none focus:ring-1 ${
              error
                ? 'border-danger-500 focus:border-danger-500 focus:ring-danger-500'
                : 'border-secondary-300 focus:border-primary-500 focus:ring-primary-500'
            }`}
          >
            <span className={selected ? 'text-secondary-900' : 'text-secondary-400'}>
              {selected?.label || placeholder || 'Select...'}
            </span>
            <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
              <ChevronUpDownIcon className="h-5 w-5 text-secondary-400" />
            </span>
          </ListboxButton>
          <ListboxOptions
            className="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-sm shadow-lg ring-1 ring-black/5 focus:outline-none"
            transition
          >
            {options.map((option) => (
              <ListboxOption
                key={option.value}
                value={option.value}
                className="group relative cursor-default select-none py-2 pl-10 pr-4 text-secondary-900 data-[focus]:bg-primary-600 data-[focus]:text-white"
              >
                <span className="block truncate font-normal group-data-[selected]:font-semibold">
                  {option.label}
                </span>
                <span className="absolute inset-y-0 left-0 hidden items-center pl-3 text-primary-600 group-data-[selected]:flex group-data-[focus]:text-white">
                  <CheckIcon className="h-5 w-5" />
                </span>
              </ListboxOption>
            ))}
          </ListboxOptions>
        </div>
      </Listbox>
      {error && <p className="mt-1 text-sm text-danger-600">{error}</p>}
    </div>
  )
}
