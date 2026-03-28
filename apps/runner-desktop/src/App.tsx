import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SettingsPage from "./pages/SettingsPage";
import SetupWizard from "./pages/SetupWizard";
import UninstallPage from "./pages/UninstallPage";

function LoadingSpinner() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-900">
      <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
    </div>
  );
}

type Page = "loading" | "wizard" | "settings" | "uninstall";

function App() {
  const [page, setPage] = useState<Page>("loading");

  // Check initial state: first run or hash-based navigation.
  useEffect(() => {
    // If the URL hash is #uninstall (set by tray menu), go directly there.
    if (window.location.hash === "#uninstall") {
      setPage("uninstall");
      return;
    }

    invoke<boolean>("is_first_run")
      .then((result) => setPage(result ? "wizard" : "settings"))
      .catch((err) => {
        console.error("Failed to check first run:", err);
        setPage("settings");
      });
  }, []);

  // Listen for hash changes so the tray menu can navigate us at runtime.
  useEffect(() => {
    function onHashChange() {
      if (window.location.hash === "#uninstall") {
        setPage("uninstall");
      }
    }
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  if (page === "loading") return <LoadingSpinner />;
  if (page === "wizard")
    return <SetupWizard onComplete={() => setPage("settings")} />;
  if (page === "uninstall")
    return (
      <UninstallPage
        onCancel={() => {
          window.location.hash = "";
          setPage("settings");
        }}
      />
    );
  return <SettingsPage />;
}

export default App;
