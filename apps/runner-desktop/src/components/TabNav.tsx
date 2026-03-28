interface Tab {
  id: string;
  label: string;
}

interface TabNavProps {
  tabs: Tab[];
  activeTab: string;
  onChange: (tabId: string) => void;
}

export default function TabNav({ tabs, activeTab, onChange }: TabNavProps) {
  return (
    <nav className="flex border-b border-gray-700 mb-4">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onChange(tab.id)}
          className={`
            px-4 py-2 text-sm font-medium border-b-2 transition-colors
            ${
              activeTab === tab.id
                ? "border-blue-500 text-blue-400"
                : "border-transparent text-gray-400 hover:text-gray-200 hover:border-gray-600"
            }
          `}
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}
