import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import TabNav from "../components/TabNav";
import TextInput from "../components/TextInput";
import Toggle from "../components/Toggle";
import ListEditor from "../components/ListEditor";
import { type Settings, defaultSettings } from "../types";

const TABS = [
  { id: "connection", label: "Connection" },
  { id: "behavior", label: "Behavior" },
  { id: "repos", label: "Repositories" },
  { id: "security", label: "Security" },
  { id: "updates", label: "Updates" },
];

const AUTONOMY_LEVELS = ["normal", "stricter", "autonomous"] as const;

const ACTION_CATEGORIES = [
  "file_read",
  "file_write",
  "shell_execute",
  "git_operations",
  "network_access",
  "package_install",
] as const;

interface SettingsPageProps {
  onNavigateWizard?: () => void;
}

export default function SettingsPage({ onNavigateWizard: _onNavigateWizard }: SettingsPageProps) {
  const [activeTab, setActiveTab] = useState("connection");
  const [settings, setSettings] = useState<Settings>(defaultSettings());
  const [token, setToken] = useState("");
  const [showToken, setShowToken] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<
    { type: "idle" } | { type: "testing" } | { type: "success"; message: string } | { type: "error"; message: string }
  >({ type: "idle" });
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const settingsJson = await invoke<string>("get_settings");
        setSettings(JSON.parse(settingsJson));
        const tokenValue = await invoke<string>("get_token");
        setToken(tokenValue);
      } catch (err) {
        console.error("Failed to load settings:", err);
      }
    }
    load();
  }, []);

  function updateSettings(patch: Partial<Settings>) {
    setSettings((prev) => ({ ...prev, ...patch }));
    setDirty(true);
  }

  async function handleTestConnection() {
    setConnectionStatus({ type: "testing" });
    try {
      const result = await invoke<string>("test_connection", {
        url: settings.control_service_url,
      });
      setConnectionStatus({ type: "success", message: result });
    } catch (err) {
      setConnectionStatus({
        type: "error",
        message: typeof err === "string" ? err : String(err),
      });
    }
  }

  async function handleSave() {
    setSaveStatus("saving");
    try {
      await invoke("save_settings", {
        settingsJson: JSON.stringify(settings),
      });
      await invoke("set_token", { token });
      setSaveStatus("saved");
      setDirty(false);
      setTimeout(() => setSaveStatus("idle"), 2000);
    } catch (err) {
      console.error("Failed to save settings:", err);
      setSaveStatus("error");
      setTimeout(() => setSaveStatus("idle"), 3000);
    }
  }

  function handleCancel() {
    // Reload from backend
    invoke<string>("get_settings")
      .then((json) => {
        setSettings(JSON.parse(json));
        setDirty(false);
      })
      .catch(console.error);
    invoke<string>("get_token")
      .then((t) => setToken(t))
      .catch(console.error);
  }

  function toggleAutonomyLevel(level: string) {
    const levels = settings.allowed_autonomy_levels;
    if (levels.includes(level)) {
      updateSettings({
        allowed_autonomy_levels: levels.filter((l) => l !== level),
      });
    } else {
      updateSettings({
        allowed_autonomy_levels: [...levels, level],
      });
    }
  }

  function toggleActionCategory(category: string) {
    const blocked = settings.blocked_action_categories;
    if (blocked.includes(category)) {
      updateSettings({
        blocked_action_categories: blocked.filter((c) => c !== category),
      });
    } else {
      updateSettings({
        blocked_action_categories: [...blocked, category],
      });
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-gray-900 text-white">
      {/* Header */}
      <div className="border-b border-gray-800 px-6 py-4">
        <h1 className="text-lg font-semibold">Cadre Machine Runner</h1>
        <p className="text-sm text-gray-400">Settings</p>
      </div>

      {/* Tabs */}
      <div className="px-6 pt-4">
        <TabNav tabs={TABS} activeTab={activeTab} onChange={setActiveTab} />
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto px-6 pb-4">
        <div className="max-w-lg space-y-4">
          {activeTab === "connection" && (
            <>
              <TextInput
                label="Server URL"
                value={settings.control_service_url}
                onChange={(v) => updateSettings({ control_service_url: v })}
                placeholder="https://cadre.example.com"
                type="url"
              />
              <TextInput
                label="Machine Name"
                value={settings.machine_name}
                onChange={(v) => updateSettings({ machine_name: v })}
                placeholder="my-machine"
              />
              <div className="space-y-1">
                <label className="block text-sm font-medium text-gray-200">
                  API Token
                </label>
                <div className="flex items-center gap-2">
                  <input
                    type={showToken ? "text" : "password"}
                    value={token}
                    onChange={(e) => {
                      setToken(e.target.value);
                      setDirty(true);
                    }}
                    placeholder="tok_..."
                    className="flex-1 rounded-md border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                  <button
                    type="button"
                    onClick={() => setShowToken(!showToken)}
                    className="rounded-md bg-gray-700 px-2 py-1.5 text-gray-400 hover:text-gray-200 transition-colors"
                    aria-label={showToken ? "Hide token" : "Show token"}
                  >
                    {showToken ? (
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
                      </svg>
                    ) : (
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                      </svg>
                    )}
                  </button>
                </div>
              </div>
              <div>
                <button
                  type="button"
                  onClick={handleTestConnection}
                  disabled={connectionStatus.type === "testing"}
                  className="rounded-md bg-gray-700 px-4 py-1.5 text-sm text-gray-200 hover:bg-gray-600 disabled:opacity-50 transition-colors"
                >
                  {connectionStatus.type === "testing"
                    ? "Testing..."
                    : "Test Connection"}
                </button>
                {connectionStatus.type === "success" && (
                  <p className="mt-1 text-sm text-green-400">
                    {connectionStatus.message}
                  </p>
                )}
                {connectionStatus.type === "error" && (
                  <p className="mt-1 text-sm text-red-400">
                    {connectionStatus.message}
                  </p>
                )}
              </div>
            </>
          )}

          {activeTab === "behavior" && (
            <>
              <Toggle
                label="Auto-start on login"
                description="Start the runner automatically when you log in"
                checked={settings.auto_start}
                onChange={(v) => updateSettings({ auto_start: v })}
              />
              <Toggle
                label="Start minimized"
                description="Hide the window on startup, run in the system tray"
                checked={settings.start_minimized}
                onChange={(v) => updateSettings({ start_minimized: v })}
              />
              <Toggle
                label="Enabled"
                description="Master switch -- when disabled, the runner pauses heartbeats"
                checked={settings.enabled}
                onChange={(v) => updateSettings({ enabled: v })}
              />
              <div className="space-y-1">
                <label className="block text-sm font-medium text-gray-200">
                  Heartbeat Interval
                </label>
                <p className="text-xs text-gray-500">
                  How often to check in with the server ({settings.heartbeat_interval_secs}s)
                </p>
                <input
                  type="range"
                  min={15}
                  max={120}
                  step={5}
                  value={settings.heartbeat_interval_secs}
                  onChange={(e) =>
                    updateSettings({
                      heartbeat_interval_secs: Number(e.target.value),
                    })
                  }
                  className="w-full accent-blue-500"
                />
                <div className="flex justify-between text-xs text-gray-500">
                  <span>15s</span>
                  <span>120s</span>
                </div>
              </div>
              <TextInput
                label="Max Concurrent Sessions"
                description="Maximum number of Claude sessions running at once"
                value={String(settings.max_concurrent_sessions)}
                onChange={(v) => {
                  const n = parseInt(v, 10);
                  if (!isNaN(n) && n >= 1 && n <= 10) {
                    updateSettings({ max_concurrent_sessions: n });
                  }
                }}
                type="number"
                min={1}
                max={10}
              />
            </>
          )}

          {activeTab === "repos" && (
            <>
              <ListEditor
                label="Repository Directories"
                description="Directories to scan for git repositories"
                items={settings.repo_directories}
                onChange={(v) => updateSettings({ repo_directories: v })}
                addLabel="Add Directory"
                placeholder="/path/to/projects"
              />
              <Toggle
                label="Restrict to allowed repos only"
                description="When enabled, only repos in the allowed list can be used"
                checked={settings.restrict_to_repos}
                onChange={(v) => updateSettings({ restrict_to_repos: v })}
              />
              <ListEditor
                label="Allowed Repos"
                description="Repository names explicitly allowed (empty = all allowed)"
                items={settings.allowed_repos}
                onChange={(v) => updateSettings({ allowed_repos: v })}
                addLabel="Add"
                placeholder="owner/repo-name"
              />
              <ListEditor
                label="Blocked Repos"
                description="Repository names that are blocked"
                items={settings.blocked_repos}
                onChange={(v) => updateSettings({ blocked_repos: v })}
                addLabel="Add"
                placeholder="owner/repo-name"
              />
            </>
          )}

          {activeTab === "security" && (
            <>
              <Toggle
                label="Local approval required"
                description="Require manual approval before executing sessions on this machine"
                checked={settings.local_approval_required}
                onChange={(v) =>
                  updateSettings({ local_approval_required: v })
                }
              />
              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-200">
                  Allowed Autonomy Levels
                </label>
                <p className="text-xs text-gray-500">
                  Which autonomy levels are permitted on this machine
                </p>
                <div className="space-y-1">
                  {AUTONOMY_LEVELS.map((level) => (
                    <label
                      key={level}
                      className="flex items-center gap-2 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={settings.allowed_autonomy_levels.includes(level)}
                        onChange={() => toggleAutonomyLevel(level)}
                        className="rounded border-gray-600 bg-gray-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-gray-900"
                      />
                      <span className="text-sm text-gray-200 capitalize">
                        {level}
                      </span>
                    </label>
                  ))}
                </div>
              </div>
              <Toggle
                label="Block autonomous mode"
                description="Override: block autonomous mode regardless of server policy"
                checked={settings.block_autonomous_mode}
                onChange={(v) =>
                  updateSettings({ block_autonomous_mode: v })
                }
              />
              <TextInput
                label="Session Timeout"
                description="Maximum session duration (0 = no limit)"
                value={String(settings.session_timeout_minutes)}
                onChange={(v) => {
                  const n = parseInt(v, 10);
                  if (!isNaN(n) && n >= 0) {
                    updateSettings({ session_timeout_minutes: n });
                  }
                }}
                type="number"
                suffix="minutes"
                min={0}
              />
              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-200">
                  Action Categories
                </label>
                <p className="text-xs text-gray-500">
                  Toggle which action categories are allowed on this machine
                </p>
                <div className="space-y-1">
                  {ACTION_CATEGORIES.map((category) => {
                    const isBlocked =
                      settings.blocked_action_categories.includes(category);
                    return (
                      <div
                        key={category}
                        className="flex items-center justify-between rounded bg-gray-800 px-3 py-1.5"
                      >
                        <span className="text-sm text-gray-200">
                          {category.replace(/_/g, " ")}
                        </span>
                        <button
                          type="button"
                          onClick={() => toggleActionCategory(category)}
                          className={`text-xs px-2 py-0.5 rounded ${
                            isBlocked
                              ? "bg-red-900/50 text-red-400 hover:bg-red-900/70"
                              : "bg-green-900/50 text-green-400 hover:bg-green-900/70"
                          } transition-colors`}
                        >
                          {isBlocked ? "Blocked" : "Allowed"}
                        </button>
                      </div>
                    );
                  })}
                </div>
              </div>
            </>
          )}

          {activeTab === "updates" && (
            <>
              <Toggle
                label="Auto-update"
                description="Automatically install new versions of the runner"
                checked={settings.auto_update}
                onChange={(v) => updateSettings({ auto_update: v })}
              />
              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-200">
                  Update Channel
                </label>
                <div className="flex gap-4">
                  {["stable", "beta"].map((channel) => (
                    <label
                      key={channel}
                      className="flex items-center gap-2 cursor-pointer"
                    >
                      <input
                        type="radio"
                        name="update_channel"
                        value={channel}
                        checked={settings.update_channel === channel}
                        onChange={() =>
                          updateSettings({ update_channel: channel })
                        }
                        className="border-gray-600 bg-gray-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-gray-900"
                      />
                      <span className="text-sm text-gray-200 capitalize">
                        {channel}
                      </span>
                    </label>
                  ))}
                </div>
              </div>
              <div className="rounded bg-gray-800 px-4 py-3">
                <p className="text-sm text-gray-400">
                  Current version:{" "}
                  <span className="text-gray-200 font-mono">0.1.0</span>
                </p>
              </div>
              <button
                type="button"
                className="rounded-md bg-gray-700 px-4 py-1.5 text-sm text-gray-200 hover:bg-gray-600 transition-colors"
              >
                Check for Updates
              </button>
            </>
          )}
        </div>
      </div>

      {/* Footer */}
      <div className="border-t border-gray-800 px-6 py-3 flex items-center justify-between">
        <div className="text-sm">
          {saveStatus === "saved" && (
            <span className="text-green-400">Settings saved</span>
          )}
          {saveStatus === "error" && (
            <span className="text-red-400">Failed to save settings</span>
          )}
          {saveStatus === "idle" && dirty && (
            <span className="text-yellow-400">Unsaved changes</span>
          )}
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={handleCancel}
            disabled={!dirty || saveStatus === "saving"}
            className="rounded-md border border-gray-700 px-4 py-1.5 text-sm text-gray-300 hover:bg-gray-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={!dirty || saveStatus === "saving"}
            className="rounded-md bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {saveStatus === "saving" ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
