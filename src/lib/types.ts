export interface CurrentSummoner {
  displayName: string;
  gameName: string;
  tagLine: string;
  summonerId: number;
  accountId: number;
  profileIconId: number;
  summonerLevel: number;
  puuid: string;
}

export interface ConnectionStatus {
  connected: boolean;
  summoner: CurrentSummoner | null;
  phase: string | null;
}

export interface RoleChampions {
  default: string[];
  top: string[];
  jungle: string[];
  middle: string[];
  bottom: string[];
  utility: string[];
}

export type RoleKey = keyof RoleChampions;

export interface RoleSpells {
  default: [string, string];
  top: [string, string];
  jungle: [string, string];
  middle: [string, string];
  bottom: [string, string];
  utility: [string, string];
}

export interface InstalockCfg {
  enabled: boolean;
  prepick: boolean;
  champions: RoleChampions;
  delay_seconds: number;
}

export interface AutobanCfg {
  enabled: boolean;
  champions: RoleChampions;
  delay_seconds: number;
}

export interface AutoRunesCfg {
  enabled: boolean;
}

export interface AutoSpellsCfg {
  enabled: boolean;
  roles: RoleSpells;
}

export interface RunePage {
  name: string;
  primary_style_id: number;
  sub_style_id: number;
  selected_perk_ids: number[];
}

export interface CustomRunesCfg {
  pages: RunePage[];
}

export interface Perk {
  id: number;
  name: string;
  desc: string;
  icon: string;
}

export interface RuneSlot {
  type: string;
  perks: number[];
}

export interface RuneTree {
  id: number;
  name: string;
  icon: string;
  slots: RuneSlot[];
}

export interface RuneData {
  trees: RuneTree[];
  perks: Record<string, Perk>;
  shards: number[][];
}

export interface AutoAcceptCfg {
  enabled: boolean;
  delay_seconds: number;
}

export interface LobbyRevealCfg {
  provider: string;
}

export interface Config {
  instalock: InstalockCfg;
  autoban: AutobanCfg;
  auto_accept: AutoAcceptCfg;
  lobby_reveal: LobbyRevealCfg;
  auto_runes: AutoRunesCfg;
  auto_spells: AutoSpellsCfg;
  custom_runes: CustomRunesCfg;
}

export interface ChampionInfo {
  id: number;
  name: string;
  alias: string;
}

export type AutomationName =
  | "auto_accept"
  | "instalock"
  | "prepick"
  | "autoban"
  | "auto_runes"
  | "auto_spells";

export interface AccountMeta {
  id: string;
  label: string;
  gameName: string;
  tagLine: string;
  puuid: string;
  region: string;
  profileIconId: number;
  createdMs: number;
}

export type ViewId =
  | "switcher"
  | "automation"
  | "customization"
  | "runes"
  | "tools"
  | "social"
  | "settings";
