import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface UninstallPageProps {
  onCancel: () => void;
}

export default function UninstallPage({ onCancel }: UninstallPageProps) {
  const [deregister, setDeregister] = useState(true);
  const [status, setStatus] = useState<"idle" | "uninstalling" | "done">("idle");

  async function handleUninstall() {
    setStatus("uninstalling");
    try {
      await invoke("uninstall", { deregister });
      setStatus("done");
    } catch (err) {
      console.error("Uninstall failed:", err);
      // The app should be exiting regardless; if we get here
      // something unexpected happened.
      setStatus("done");
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-gray-900 text-white">
      {/* Header */}
      <div className="border-b border-gray-800 px-6 py-4">
        <h1 className="text-lg font-semibold">Cadre Machine Runner</h1>
        <p className="text-sm text-gray-400">Uninstall</p>
      </div>

      {/* Content */}
      <div className="flex-1 px-6 py-6">
        <div className="max-w-lg space-y-6">
          {status === "idle" && (
            <>
              {/* Warning */}
              <div className="rounded-md border border-red-800 bg-red-950/50 px-4 py-3">
                <p className="text-sm font-medium text-red-400">
                  This will remove Cadre Machine Runner from your system.
                </p>
              </div>

              {/* What will be removed */}
              <div className="space-y-2">
                <h2 className="text-sm font-medium text-gray-200">
                  The following will be removed:
                </h2>
                <ul className="list-disc pl-5 space-y-1 text-sm text-gray-400">
                  <li>Settings file (~/.config/cadre/settings.json)</li>
                  <li>Keychain credentials for this machine</li>
                  <li>Auto-start login entry</li>
                  <li>Log files</li>
                </ul>
              </div>

              {/* Deregister checkbox */}
              <label className="flex items-start gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={deregister}
                  onChange={(e) => setDeregister(e.target.checked)}
                  className="mt-0.5 rounded border-gray-600 bg-gray-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-gray-900"
                />
                <div>
                  <span className="text-sm font-medium text-gray-200">
                    Also deregister this machine from the server
                  </span>
                  <p className="text-xs text-gray-500 mt-0.5">
                    Revokes this machine on the control service so it can no
                    longer receive sessions.
                  </p>
                </div>
              </label>

              {/* Buttons */}
              <div className="flex items-center gap-3 pt-2">
                <button
                  type="button"
                  onClick={handleUninstall}
                  className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-500 transition-colors"
                >
                  Uninstall
                </button>
                <button
                  type="button"
                  onClick={onCancel}
                  className="rounded-md border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors"
                >
                  Cancel
                </button>
              </div>
            </>
          )}

          {status === "uninstalling" && (
            <div className="flex flex-col items-center justify-center py-12 space-y-4">
              <div className="w-8 h-8 border-4 border-red-500 border-t-transparent rounded-full animate-spin" />
              <p className="text-sm text-gray-400">Uninstalling...</p>
            </div>
          )}

          {status === "done" && (
            <div className="flex flex-col items-center justify-center py-12 space-y-2">
              <p className="text-sm text-gray-200">Uninstall complete.</p>
              <p className="text-xs text-gray-500">
                The application will close shortly.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
