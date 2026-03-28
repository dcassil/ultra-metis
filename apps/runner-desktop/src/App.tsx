import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SettingsPage from "./pages/SettingsPage";
import SetupWizard from "./pages/SetupWizard";

function LoadingSpinner() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-900">
      <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
    </div>
  );
}

function App() {
  const [isFirstRun, setIsFirstRun] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<boolean>("is_first_run")
      .then((result) => setIsFirstRun(result))
      .catch((err) => {
        console.error("Failed to check first run:", err);
        setIsFirstRun(false); // Fall back to settings page on error
      });
  }, []);

  if (isFirstRun === null) return <LoadingSpinner />;
  if (isFirstRun)
    return <SetupWizard onComplete={() => setIsFirstRun(false)} />;
  return <SettingsPage />;
}

export default App;
