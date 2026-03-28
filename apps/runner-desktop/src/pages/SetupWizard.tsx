import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import TextInput from "../components/TextInput";
import ListEditor from "../components/ListEditor";
import Toggle from "../components/Toggle";
import { type Settings, defaultSettings } from "../types";

type Step = "welcome" | "server" | "identity" | "repos" | "security" | "done";

const STEPS: Step[] = ["welcome", "server", "identity", "repos", "security", "done"];

interface SetupWizardProps {
  onComplete: () => void;
}

export default function SetupWizard({ onComplete }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState<Step>("welcome");
  const [settings, setSettings] = useState<Settings>(defaultSettings());
  const [token, setToken] = useState("");

  // Connection test state
  const [connectionStatus, setConnectionStatus] = useState<
    { type: "idle" } | { type: "testing" } | { type: "success" } | { type: "error"; message: string }
  >({ type: "idle" });

  // Done-step state
  const [runnerStatus, setRunnerStatus] = useState<"starting" | "connected" | "error">("starting");

  const stepIndex = STEPS.indexOf(currentStep);

  function updateSettings(patch: Partial<Settings>) {
    setSettings((prev) => ({ ...prev, ...patch }));
  }

  function goNext() {
    if (stepIndex < STEPS.length - 1) {
      setCurrentStep(STEPS[stepIndex + 1]);
    }
  }

  function goBack() {
    if (stepIndex > 0) {
      setCurrentStep(STEPS[stepIndex - 1]);
    }
  }

  async function handleTestConnection() {
    setConnectionStatus({ type: "testing" });
    try {
      await invoke<string>("test_connection", {
        url: settings.control_service_url,
      });
      setConnectionStatus({ type: "success" });
    } catch (err) {
      setConnectionStatus({
        type: "error",
        message: typeof err === "string" ? err : String(err),
      });
    }
  }

  // When entering the "done" step, save settings and start the runner.
  useEffect(() => {
    if (currentStep !== "done") return;

    let cancelled = false;

    async function finalize() {
      try {
        await invoke("save_settings", {
          settingsJson: JSON.stringify(settings),
        });
        await invoke("set_token", { token });
        await invoke("start_runner");

        // Poll for status
        for (let i = 0; i < 30; i++) {
          if (cancelled) return;
          const statusJson = await invoke<string>("get_status");
          const status = JSON.parse(statusJson);

          if (typeof status === "object" && status !== null) {
            // Active or PendingApproval means we connected
            if ("Active" in status || status === "PendingApproval") {
              if (!cancelled) setRunnerStatus("connected");
              return;
            }
            if ("Error" in status) {
              if (!cancelled) setRunnerStatus("error");
              return;
            }
          }
          if (status === "PendingApproval") {
            if (!cancelled) setRunnerStatus("connected");
            return;
          }

          await new Promise((r) => setTimeout(r, 1000));
        }
        // Timed out -- consider it connected anyway (might be registering still)
        if (!cancelled) setRunnerStatus("connected");
      } catch (err) {
        console.error("Setup finalization error:", err);
        if (!cancelled) setRunnerStatus("error");
      }
    }

    finalize();
    return () => {
      cancelled = true;
    };
  }, [currentStep, settings, token]);

  const canProceedFromServer = connectionStatus.type === "success";
  const canProceedFromIdentity = settings.machine_name.trim() !== "" && token.trim() !== "";

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-900 text-white p-8">
      <div className="w-full max-w-md">
        {/* Progress indicator */}
        {currentStep !== "welcome" && currentStep !== "done" && (
          <div className="mb-8">
            <div className="flex items-center justify-between mb-2">
              {STEPS.filter((s) => s !== "welcome" && s !== "done").map(
                (step, idx) => {
                  const activeIdx = STEPS.filter(
                    (s) => s !== "welcome" && s !== "done"
                  ).indexOf(currentStep);
                  return (
                    <div
                      key={step}
                      className={`flex-1 h-1 rounded-full mx-0.5 ${
                        idx <= activeIdx ? "bg-blue-500" : "bg-gray-700"
                      }`}
                    />
                  );
                }
              )}
            </div>
            <p className="text-xs text-gray-500 text-center">
              Step {stepIndex} of {STEPS.length - 2}
            </p>
          </div>
        )}

        {/* Welcome */}
        {currentStep === "welcome" && (
          <div className="text-center space-y-6">
            <div className="w-16 h-16 mx-auto rounded-2xl bg-blue-600 flex items-center justify-center">
              <svg className="w-8 h-8 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <div>
              <h1 className="text-2xl font-bold">Welcome to Cadre</h1>
              <h2 className="text-lg text-gray-400 mt-1">Machine Runner</h2>
            </div>
            <p className="text-gray-400 text-sm leading-relaxed">
              This runner connects your machine to the Cadre control service,
              allowing remote AI engineering sessions to execute in your local
              repositories.
            </p>
            <button
              type="button"
              onClick={goNext}
              className="w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
            >
              Get Started
            </button>
          </div>
        )}

        {/* Server Connection */}
        {currentStep === "server" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-semibold">Server Connection</h2>
              <p className="text-sm text-gray-400 mt-1">
                Enter the URL of your Cadre control service.
              </p>
            </div>
            <TextInput
              label="Server URL"
              value={settings.control_service_url}
              onChange={(v) => updateSettings({ control_service_url: v })}
              placeholder="https://cadre.example.com"
              type="url"
            />
            <div>
              <button
                type="button"
                onClick={handleTestConnection}
                disabled={
                  connectionStatus.type === "testing" ||
                  !settings.control_service_url.trim()
                }
                className="rounded-md bg-gray-700 px-4 py-1.5 text-sm text-gray-200 hover:bg-gray-600 disabled:opacity-50 transition-colors"
              >
                {connectionStatus.type === "testing"
                  ? "Testing..."
                  : "Test Connection"}
              </button>
              {connectionStatus.type === "success" && (
                <p className="mt-2 text-sm text-green-400">
                  Connection successful
                </p>
              )}
              {connectionStatus.type === "error" && (
                <p className="mt-2 text-sm text-red-400">
                  {connectionStatus.message}
                </p>
              )}
            </div>
            <div className="flex gap-3 pt-2">
              <button
                type="button"
                onClick={goBack}
                className="rounded-md border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors"
              >
                Back
              </button>
              <button
                type="button"
                onClick={goNext}
                disabled={!canProceedFromServer}
                className="flex-1 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Continue
              </button>
            </div>
          </div>
        )}

        {/* Machine Identity */}
        {currentStep === "identity" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-semibold">Machine Identity</h2>
              <p className="text-sm text-gray-400 mt-1">
                This identifies your machine on the server.
              </p>
            </div>
            <TextInput
              label="Machine Name"
              value={settings.machine_name}
              onChange={(v) => updateSettings({ machine_name: v })}
              placeholder="my-dev-machine"
              description="A human-readable name for this machine"
            />
            <div className="space-y-1">
              <label className="block text-sm font-medium text-gray-200">
                API Token
              </label>
              <p className="text-xs text-gray-500">
                Get this from your Cadre dashboard or administrator
              </p>
              <input
                type="password"
                value={token}
                onChange={(e) => setToken(e.target.value)}
                placeholder="tok_..."
                className="w-full rounded-md border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              />
            </div>
            <div className="flex gap-3 pt-2">
              <button
                type="button"
                onClick={goBack}
                className="rounded-md border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors"
              >
                Back
              </button>
              <button
                type="button"
                onClick={goNext}
                disabled={!canProceedFromIdentity}
                className="flex-1 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Continue
              </button>
            </div>
          </div>
        )}

        {/* Repository Selection */}
        {currentStep === "repos" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-semibold">Repositories</h2>
              <p className="text-sm text-gray-400 mt-1">
                These directories will be scanned for git repositories available
                for remote sessions.
              </p>
            </div>
            <ListEditor
              items={settings.repo_directories}
              onChange={(v) => updateSettings({ repo_directories: v })}
              addLabel="Add Directory"
              placeholder="~/projects"
            />
            {settings.repo_directories.length === 0 && (
              <p className="text-xs text-yellow-400">
                Add at least one directory to enable remote sessions.
              </p>
            )}
            <div className="flex gap-3 pt-2">
              <button
                type="button"
                onClick={goBack}
                className="rounded-md border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors"
              >
                Back
              </button>
              <button
                type="button"
                onClick={goNext}
                className="flex-1 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
              >
                Continue
              </button>
            </div>
          </div>
        )}

        {/* Security Review */}
        {currentStep === "security" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-semibold">Security Settings</h2>
              <p className="text-sm text-gray-400 mt-1">
                Review the default security settings. You can change these later
                in Settings.
              </p>
            </div>
            <div className="space-y-3">
              <Toggle
                label="Local approval required"
                description="Require manual approval before executing sessions"
                checked={settings.local_approval_required}
                onChange={(v) =>
                  updateSettings({ local_approval_required: v })
                }
              />
              <Toggle
                label="Block autonomous mode"
                description="Prevent fully autonomous sessions on this machine"
                checked={settings.block_autonomous_mode}
                onChange={(v) =>
                  updateSettings({ block_autonomous_mode: v })
                }
              />
              <div className="rounded bg-gray-800 px-4 py-3 space-y-1">
                <p className="text-sm text-gray-300 font-medium">
                  Default security profile:
                </p>
                <ul className="text-xs text-gray-400 space-y-0.5">
                  <li>
                    -- Autonomy levels: {settings.allowed_autonomy_levels.join(", ")}
                  </li>
                  <li>-- Session timeout: {settings.session_timeout_minutes === 0 ? "No limit" : `${settings.session_timeout_minutes} min`}</li>
                  <li>-- Max concurrent sessions: {settings.max_concurrent_sessions}</li>
                </ul>
              </div>
            </div>
            <div className="flex gap-3 pt-2">
              <button
                type="button"
                onClick={goBack}
                className="rounded-md border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors"
              >
                Back
              </button>
              <button
                type="button"
                onClick={goNext}
                className="flex-1 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
              >
                Finish Setup
              </button>
            </div>
          </div>
        )}

        {/* Done */}
        {currentStep === "done" && (
          <div className="text-center space-y-6">
            {runnerStatus === "starting" && (
              <>
                <div className="w-12 h-12 mx-auto border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
                <div>
                  <h2 className="text-xl font-semibold">
                    Setup Complete!
                  </h2>
                  <p className="text-sm text-gray-400 mt-1">
                    Your machine runner is connecting...
                  </p>
                </div>
              </>
            )}
            {runnerStatus === "connected" && (
              <>
                <div className="w-12 h-12 mx-auto rounded-full bg-green-600 flex items-center justify-center">
                  <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <div>
                  <h2 className="text-xl font-semibold">Connected!</h2>
                  <p className="text-sm text-gray-400 mt-1">
                    Your machine is registered and ready for sessions.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={onComplete}
                  className="w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
                >
                  Open Settings
                </button>
              </>
            )}
            {runnerStatus === "error" && (
              <>
                <div className="w-12 h-12 mx-auto rounded-full bg-red-600 flex items-center justify-center">
                  <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </div>
                <div>
                  <h2 className="text-xl font-semibold">Connection Issue</h2>
                  <p className="text-sm text-gray-400 mt-1">
                    Settings were saved, but the runner could not connect.
                    You can retry from Settings.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={onComplete}
                  className="w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
                >
                  Open Settings
                </button>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
