const BASE =
  "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default";

export const profileIconUrl = (id: number) => `${BASE}/v1/profile-icons/${id}.jpg`;

export const championIconUrl = (id: number) =>
  `${BASE}/v1/champion-icons/${id}.png`;

export const SPELL_ID: Record<string, number> = {
  Flash: 4,
  Smite: 11,
  Teleport: 12,
  Ignite: 14,
  Heal: 7,
  Ghost: 6,
  Exhaust: 3,
  Barrier: 21,
  Cleanse: 1,
  Snowball: 32,
  Clarity: 13,
};

let spellIconsCache: Map<number, string> | null = null;

export async function fetchSummonerSpellIcons(): Promise<Map<number, string>> {
  if (spellIconsCache) return spellIconsCache;
  const res = await fetch(`${BASE}/v1/summoner-spells.json`);
  if (!res.ok) throw new Error(`CDragon returned HTTP ${res.status}`);
  const raw: { id: number; iconPath?: string }[] = await res.json();
  const map = new Map<number, string>();
  for (const s of raw) {
    if (s.iconPath) map.set(s.id, assetUrl(s.iconPath));
  }
  spellIconsCache = map;
  return map;
}

export interface SkinInfo {
  id: number;
  name: string;
  tileUrl: string;
}

function assetUrl(gameDataPath: string): string {
  return BASE + gameDataPath.replace(/^\/lol-game-data\/assets/i, "").toLowerCase();
}

let skinsCache: SkinInfo[] | null = null;

export async function fetchSkins(): Promise<SkinInfo[]> {
  if (skinsCache) return skinsCache;
  const res = await fetch(`${BASE}/v1/skins.json`);
  if (!res.ok) throw new Error(`CDragon returned HTTP ${res.status}`);
  const raw: Record<string, { id: number; name: string; tilePath?: string }> =
    await res.json();
  skinsCache = Object.values(raw)
    .filter((s) => s.tilePath)
    .map((s) => ({ id: s.id, name: s.name, tileUrl: assetUrl(s.tilePath!) }));
  return skinsCache;
}
