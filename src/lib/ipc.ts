import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AccountMeta,
  AutomationName,
  ChampionInfo,
  Config,
  ConnectionStatus,
  RuneData,
  RunePage,
  UpdateInfo,
  UpdateProgress,
} from "./types";

export function getConnectionStatus(): Promise<ConnectionStatus> {
  return invoke("get_connection_status");
}

export function getConfig(): Promise<Config> {
  return invoke("get_config");
}

export function setConfig(config: Config): Promise<void> {
  return invoke("set_config", { config });
}

export function listAccounts(): Promise<AccountMeta[]> {
  return invoke("list_accounts");
}

export function saveCurrentAccount(label?: string): Promise<AccountMeta> {
  return invoke("save_current_account", { label: label ?? null });
}

export function switchAccount(id: string): Promise<void> {
  return invoke("switch_account", { id });
}

export function renameAccount(id: string, label: string): Promise<AccountMeta> {
  return invoke("rename_account", { id, label });
}

export function deleteAccount(id: string): Promise<void> {
  return invoke("delete_account", { id });
}

export function setProfileIcon(iconId: number): Promise<void> {
  return invoke("set_profile_icon", { iconId });
}

export function setClientIcon(iconId: number): Promise<void> {
  return invoke("set_client_icon", { iconId });
}

export function setBackground(skinId: number): Promise<void> {
  return invoke("set_background", { skinId });
}

export function setStatusMessage(message: string): Promise<void> {
  return invoke("set_status_message", { message });
}

export type BadgeMode = "clear" | "glitch";

export function setBadges(mode: BadgeMode): Promise<void> {
  return invoke("set_badges", { mode });
}

export function revealLobby(): Promise<string> {
  return invoke("reveal_lobby");
}

export function dodge(): Promise<void> {
  return invoke("dodge");
}

export function restartUx(): Promise<void> {
  return invoke("restart_ux");
}

export function setChatOffline(offline: boolean): Promise<void> {
  return invoke("set_chat_offline", { offline });
}

export function countFriends(): Promise<number> {
  return invoke("count_friends");
}

export function removeAllFriends(): Promise<number> {
  return invoke("remove_all_friends");
}

export function setAutomation(
  name: AutomationName,
  enabled: boolean,
): Promise<void> {
  return invoke("set_automation", { name, enabled });
}

export function getRunningAutomations(): Promise<string[]> {
  return invoke("get_running_automations");
}

export function listChampions(): Promise<ChampionInfo[]> {
  return invoke("list_champions");
}

export function getRuneTrees(): Promise<RuneData> {
  return invoke("get_rune_trees");
}

export function applyRunePage(page: RunePage): Promise<void> {
  return invoke("apply_rune_page", { page });
}

export function onAutomations(
  cb: (running: string[]) => void,
): Promise<UnlistenFn> {
  return listen<string[]>("prowler://automations", (e) => cb(e.payload));
}

export function onStatus(cb: (s: ConnectionStatus) => void): Promise<UnlistenFn> {
  return listen<ConnectionStatus>("lcu://status", (e) => cb(e.payload));
}

export function onConfig(cb: (c: Config) => void): Promise<UnlistenFn> {
  return listen<Config>("prowler://config", (e) => cb(e.payload));
}

export function onLog(cb: (line: string) => void): Promise<UnlistenFn> {
  return listen<string>("prowler://log", (e) => cb(e.payload));
}

export function checkUpdate(): Promise<UpdateInfo | null> {
  return invoke("check_update");
}

export function installUpdate(): Promise<void> {
  return invoke("install_update");
}

export function onUpdate(
  cb: (info: UpdateInfo | null) => void,
): Promise<UnlistenFn> {
  return listen<UpdateInfo | null>("prowler://update", (e) => cb(e.payload));
}

export function onUpdateProgress(
  cb: (p: UpdateProgress) => void,
): Promise<UnlistenFn> {
  return listen<UpdateProgress>("prowler://update-progress", (e) =>
    cb(e.payload),
  );
}
