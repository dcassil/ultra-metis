interface TextInputProps {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  placeholder?: string;
  type?: "text" | "password" | "number" | "url";
  disabled?: boolean;
  description?: string;
  suffix?: string;
  min?: number;
  max?: number;
}

export default function TextInput({
  value,
  onChange,
  label,
  placeholder,
  type = "text",
  disabled = false,
  description,
  suffix,
  min,
  max,
}: TextInputProps) {
  return (
    <div className="space-y-1">
      {label && (
        <label className="block text-sm font-medium text-gray-200">
          {label}
        </label>
      )}
      {description && (
        <p className="text-xs text-gray-500">{description}</p>
      )}
      <div className="flex items-center gap-2">
        <input
          type={type}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          min={min}
          max={max}
          className={`
            w-full rounded-md border border-gray-700 bg-gray-800 px-3 py-1.5
            text-sm text-gray-200 placeholder-gray-500
            focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
            ${disabled ? "opacity-50 cursor-not-allowed" : ""}
          `}
        />
        {suffix && (
          <span className="text-sm text-gray-400 whitespace-nowrap">{suffix}</span>
        )}
      </div>
    </div>
  );
}
