/** Mirrors the Rust `Settings` struct from cadre-machine-runner. */
export interface Settings {
  // Connection
  control_service_url: string;
  machine_name: string;

  // Behavior
  auto_start: boolean;
  start_minimized: boolean;
  enabled: boolean;
  heartbeat_interval_secs: number;
  max_concurrent_sessions: number;

  // Repos
  repo_directories: string[];
  allowed_repos: string[];
  blocked_repos: string[];
  restrict_to_repos: boolean;

  // Security
  local_approval_required: boolean;
  allowed_autonomy_levels: string[];
  block_autonomous_mode: boolean;
  session_timeout_minutes: number;
  allowed_action_categories: string[];
  blocked_action_categories: string[];

  // Updates
  auto_update: boolean;
  update_channel: string;

  // Logging
  log_level: string;
}

/** Default settings matching the Rust defaults. */
export function defaultSettings(): Settings {
  return {
    control_service_url: "http://localhost:8080",
    machine_name: "",
    auto_start: true,
    start_minimized: true,
    enabled: true,
    heartbeat_interval_secs: 30,
    max_concurrent_sessions: 1,
    repo_directories: [],
    allowed_repos: [],
    blocked_repos: [],
    restrict_to_repos: false,
    local_approval_required: false,
    allowed_autonomy_levels: ["normal", "stricter", "autonomous"],
    block_autonomous_mode: false,
    session_timeout_minutes: 0,
    allowed_action_categories: [],
    blocked_action_categories: [],
    auto_update: true,
    update_channel: "stable",
    log_level: "info",
  };
}
