import { forwardRef } from 'react'
import type { InputHTMLAttributes } from 'react'

interface FormInputProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string
  error?: string
}

export const FormInput = forwardRef<HTMLInputElement, FormInputProps>(
  ({ label, error, required, id, className = '', ...props }, ref) => {
    const inputId = id || label.toLowerCase().replace(/\s+/g, '-')
    return (
      <div className={className}>
        <label htmlFor={inputId} className="block text-sm font-medium text-secondary-700">
          {label}
          {required && <span className="ml-0.5 text-danger-500">*</span>}
        </label>
        <input
          ref={ref}
          id={inputId}
          required={required}
          aria-describedby={error ? `${inputId}-error` : undefined}
          aria-invalid={!!error}
          className={`mt-1 block w-full rounded-md border px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 ${
            error
              ? 'border-danger-500 focus:border-danger-500 focus:ring-danger-500'
              : 'border-secondary-300 focus:border-primary-500 focus:ring-primary-500'
          }`}
          {...props}
        />
        {error && (
          <p id={`${inputId}-error`} className="mt-1 text-sm text-danger-600">
            {error}
          </p>
        )}
      </div>
    )
  },
)

FormInput.displayName = 'FormInput'
