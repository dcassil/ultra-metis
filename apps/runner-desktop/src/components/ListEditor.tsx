import { useState } from "react";

interface ListEditorProps {
  items: string[];
  onChange: (items: string[]) => void;
  label?: string;
  description?: string;
  addLabel?: string;
  placeholder?: string;
}

export default function ListEditor({
  items,
  onChange,
  label,
  description,
  addLabel = "Add",
  placeholder = "Enter value...",
}: ListEditorProps) {
  const [newItem, setNewItem] = useState("");

  function handleAdd() {
    const trimmed = newItem.trim();
    if (trimmed && !items.includes(trimmed)) {
      onChange([...items, trimmed]);
      setNewItem("");
    }
  }

  function handleRemove(index: number) {
    onChange(items.filter((_, i) => i !== index));
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  }

  return (
    <div className="space-y-2">
      {label && (
        <label className="block text-sm font-medium text-gray-200">
          {label}
        </label>
      )}
      {description && (
        <p className="text-xs text-gray-500">{description}</p>
      )}

      {items.length > 0 && (
        <ul className="space-y-1">
          {items.map((item, index) => (
            <li
              key={index}
              className="flex items-center justify-between rounded bg-gray-800 px-3 py-1.5 text-sm text-gray-200"
            >
              <span className="truncate mr-2">{item}</span>
              <button
                type="button"
                onClick={() => handleRemove(index)}
                className="text-gray-500 hover:text-red-400 transition-colors shrink-0"
                aria-label={`Remove ${item}`}
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </li>
          ))}
        </ul>
      )}

      <div className="flex gap-2">
        <input
          type="text"
          value={newItem}
          onChange={(e) => setNewItem(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1 rounded-md border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
        <button
          type="button"
          onClick={handleAdd}
          disabled={!newItem.trim()}
          className="rounded-md bg-gray-700 px-3 py-1.5 text-sm text-gray-200 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {addLabel}
        </button>
      </div>
    </div>
  );
}
