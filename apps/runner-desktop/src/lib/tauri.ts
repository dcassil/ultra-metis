import { invoke } from "@tauri-apps/api/core";

/** Enable or disable auto-start on login. */
export async function setAutoStart(enabled: boolean): Promise<void> {
  await invoke("set_auto_start", { enabled });
}

/** Check whether auto-start on login is currently enabled. */
export async function getAutoStart(): Promise<boolean> {
  return invoke("get_auto_start");
}

/** Send a desktop notification with the given title and body. */
export async function sendNotification(
  title: string,
  body: string,
): Promise<void> {
  await invoke("send_notification", { title, body });
}
