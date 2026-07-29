import { useEffect, useRef, useState } from "react";
import {
  Ban,
  Check,
  Dices,
  Search,
  Sparkles,
  X,
  Zap,
  type LucideIcon,
} from "lucide-react";
import { Card, ViewHeader } from "../components/common";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import {
  SPELL_ID,
  championIconUrl,
  fetchSummonerSpellIcons,
} from "../lib/cdragon";
import {
  getConfig,
  listChampions,
  onStatus,
  setAutomation,
  setConfig,
} from "../lib/ipc";
import { useLog } from "../lib/log";
import type {
  AutomationName,
  ChampionInfo,
  Config,
  RoleChampions,
  RoleKey,
  RoleSpells,
} from "../lib/types";

const ROLE_TABS: { key: RoleKey; label: string }[] = [
  { key: "default", label: "Default" },
  { key: "top", label: "Top" },
  { key: "jungle", label: "Jungle" },
  { key: "middle", label: "Mid" },
  { key: "bottom", label: "Bot" },
  { key: "utility", label: "Support" },
];

const MAX_PRIORITY = 5;

type AutoTab = "instalock" | "autoban" | "auto_spells" | "auto_accept";

const AUTOMATION_TABS: { key: AutoTab; label: string; icon: LucideIcon }[] = [
  { key: "instalock", label: "Instalock", icon: Zap },
  { key: "autoban", label: "Autoban", icon: Ban },
  { key: "auto_spells", label: "Summoners", icon: Sparkles },
  { key: "auto_accept", label: "Accept", icon: Check },
];

const SUMMONER_SPELLS = [
  "Flash",
  "Smite",
  "Teleport",
  "Ignite",
  "Heal",
  "Ghost",
  "Exhaust",
  "Barrier",
  "Cleanse",
  "Snowball",
  "Clarity",
];

export function AutomationView() {
  const { log } = useLog();
  const [config, setLocal] = useState<Config | null>(null);
  const [champions, setChampions] = useState<ChampionInfo[]>([]);
  const [tab, setTab] = useState<AutoTab>("instalock");
  const championsLoaded = useRef(false);

  useEffect(() => {
    getConfig().then(setLocal).catch((e) => log(`Failed to load config: ${e}`));
  }, [log]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const load = () => {
      if (championsLoaded.current) return;
      listChampions()
        .then((list) => {
          championsLoaded.current = true;

          const seen = new Set<string>();
          setChampions(
            list.filter((c) => {
              const key = c.name.toLowerCase();
              if (seen.has(key)) return false;
              seen.add(key);
              return true;
            }),
          );
        })
        .catch(() => {});
    };
    load();
    onStatus((s) => {
      if (s.connected) load();
    }).then((fn) => (unlisten = fn));
    return () => unlisten?.();
  }, []);

  if (!config) {
    return (
      <div className="flex h-full flex-col">
        <ViewHeader title="Automation" />
        <p className="text-sm text-muted-foreground">Loading…</p>
      </div>
    );
  }

  async function toggle(name: AutomationName, enabled: boolean, next: Config) {
    setLocal(next);
    try {
      await setAutomation(name, enabled);
    } catch (e) {
      log(`Failed to toggle ${name}: ${e}`);
    }
  }

  async function save(next: Config) {
    setLocal(next);
    try {
      await setConfig(next);
    } catch (e) {
      log(`Failed to save config: ${e}`);
    }
  }

  const tabs = AUTOMATION_TABS.map((t) => {
    const enabled: Record<AutoTab, boolean> = {
      instalock: config.instalock.enabled,
      autoban: config.autoban.enabled,
      auto_spells: config.auto_spells.enabled,
      auto_accept: config.auto_accept.enabled,
    };
    return { ...t, dot: enabled[t.key] };
  });

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        title="Automation"
        subtitle="Background tasks that react to the client. Delays live in Settings."
      />

      <div className="max-w-2xl">
        <SegmentedTabs items={tabs} value={tab} onSelect={setTab} />

        <div className="mt-4">
          {tab === "instalock" && (
            <Card
              title="Instalock"
              desc="Lock your assigned role's champion; falls back down the list."
              action={
                <Switch
                  checked={config.instalock.enabled}
                  onCheckedChange={(on) =>
                    toggle("instalock", on, {
                      ...config,
                      instalock: { ...config.instalock, enabled: on },
                    })
                  }
                />
              }
            >
              <div className="grid gap-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">
                    Prepick - hover it during the ban phase
                  </span>
                  <Switch
                    checked={config.instalock.prepick}
                    onCheckedChange={(on) =>
                      toggle("prepick", on, {
                        ...config,
                        instalock: { ...config.instalock, prepick: on },
                      })
                    }
                  />
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">
                    Auto Runes - apply the recommended page after lock-in
                  </span>
                  <Switch
                    checked={config.auto_runes.enabled}
                    onCheckedChange={(on) =>
                      toggle("auto_runes", on, {
                        ...config,
                        auto_runes: { ...config.auto_runes, enabled: on },
                      })
                    }
                  />
                </div>
                <RoleChampionLists
                  value={config.instalock.champions}
                  champions={champions}
                  allowRandom
                  verb="picked"
                  onChange={(champions) =>
                    save({
                      ...config,
                      instalock: { ...config.instalock, champions },
                    })
                  }
                />
              </div>
            </Card>
          )}

          {tab === "autoban" && (
            <Card
              title="Autoban"
              desc="Ban your assigned role's target; skips champions teammates hover."
              action={
                <Switch
                  checked={config.autoban.enabled}
                  onCheckedChange={(on) =>
                    toggle("autoban", on, {
                      ...config,
                      autoban: { ...config.autoban, enabled: on },
                    })
                  }
                />
              }
            >
              <div className="grid gap-3">
                <RoleChampionLists
                  value={config.autoban.champions}
                  champions={champions}
                  allowRandom={false}
                  verb="banned"
                  onChange={(champions) =>
                    save({
                      ...config,
                      autoban: { ...config.autoban, champions },
                    })
                  }
                />
              </div>
            </Card>
          )}

          {tab === "auto_spells" && (
            <Card
              title="Auto Summoners"
              desc="Set your summoner spells for the assigned role after you lock in."
              action={
                <Switch
                  checked={config.auto_spells.enabled}
                  onCheckedChange={(on) =>
                    toggle("auto_spells", on, {
                      ...config,
                      auto_spells: { ...config.auto_spells, enabled: on },
                    })
                  }
                />
              }
            >
              <div className="grid gap-3">
                <RoleSpellPairs
                  value={config.auto_spells.roles}
                  onChange={(roles) =>
                    save({
                      ...config,
                      auto_spells: { ...config.auto_spells, roles },
                    })
                  }
                />
              </div>
            </Card>
          )}

          {tab === "auto_accept" && (
            <Card
              title="Auto Accept"
              desc="Accept the ready check as soon as a match is found."
            >
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">
                  Delay: {config.auto_accept.delay.min.toFixed(1)}–
                  {config.auto_accept.delay.max.toFixed(1)}s
                </span>
                <Switch
                  checked={config.auto_accept.enabled}
                  onCheckedChange={(on) =>
                    toggle("auto_accept", on, {
                      ...config,
                      auto_accept: { ...config.auto_accept, enabled: on },
                    })
                  }
                />
              </div>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}

function SegmentedTabs<T extends string>({
  items,
  value,
  onSelect,
  size = "md",
}: {
  items: { key: T; label: string; icon?: LucideIcon; dot?: boolean }[];
  value: T;
  onSelect: (key: T) => void;
  size?: "sm" | "md";
}) {
  return (
    <div className="flex w-full gap-1 rounded-lg border border-edge bg-background p-1">
      {items.map((item) => {
        const active = item.key === value;
        const Icon = item.icon;
        return (
          <button
            key={item.key}
            type="button"
            onClick={() => onSelect(item.key)}
            className={cn(
              "inline-flex flex-1 items-center justify-center gap-1.5 whitespace-nowrap rounded-md font-medium transition-colors",
              size === "sm" ? "px-2.5 py-1 text-xs" : "px-3 py-1.5 text-sm",
              active
                ? "bg-panel2 text-text shadow-sm"
                : "text-muted-foreground hover:text-text",
            )}
          >
            {Icon && <Icon className={size === "sm" ? "size-3.5" : "size-4"} />}
            {item.label}
            {item.dot && (
              <span className="ml-0.5 size-1.5 rounded-full bg-primary" />
            )}
          </button>
        );
      })}
    </div>
  );
}

function RoleTabs({
  role,
  onSelect,
  marked,
}: {
  role: RoleKey;
  onSelect: (r: RoleKey) => void;
  marked: (r: RoleKey) => boolean;
}) {
  return (
    <SegmentedTabs
      size="sm"
      value={role}
      onSelect={onSelect}
      items={ROLE_TABS.map((t) => ({
        key: t.key,
        label: t.label,
        dot: marked(t.key),
      }))}
    />
  );
}

function RoleChampionLists({
  value,
  champions,
  allowRandom,
  verb,
  onChange,
}: {
  value: RoleChampions;
  champions: ChampionInfo[];
  allowRandom: boolean;
  verb: string;
  onChange: (next: RoleChampions) => void;
}) {
  const [role, setRole] = useState<RoleKey>("default");
  const list = value[role];
  const set = (next: string[]) => onChange({ ...value, [role]: next });
  const full = list.length >= MAX_PRIORITY;

  const add = (name: string) => {
    if (full || list.some((n) => n.toLowerCase() === name.toLowerCase())) return;
    set([...list, name]);
  };
  const remove = (i: number) => set(list.filter((_, j) => j !== i));
  const move = (from: number, to: number) => {
    if (from === to) return;
    const next = [...list];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    set(next);
  };

  return (
    <div className="grid gap-3">
      <RoleTabs
        role={role}
        onSelect={setRole}
        marked={(r) => value[r].length > 0}
      />
      <ChampionSearch
        key={role}
        champions={champions}
        allowRandom={allowRandom}
        picked={list}
        disabled={full}
        onAdd={add}
      />
      <div className="border-t border-edge" />
      <PriorityTiles
        list={list}
        champions={champions}
        role={role}
        verb={verb}
        onRemove={remove}
        onMove={move}
      />
    </div>
  );
}

type Suggestion = { name: string; id?: number };

function ChampionSearch({
  champions,
  allowRandom,
  picked,
  disabled,
  onAdd,
}: {
  champions: ChampionInfo[];
  allowRandom: boolean;
  picked: string[];
  disabled: boolean;
  onAdd: (name: string) => void;
}) {
  const [query, setQuery] = useState("");

  const q = query.trim().toLowerCase();
  const results: Suggestion[] = [];
  if (q) {
    if (allowRandom && "random".includes(q)) results.push({ name: "Random" });
    const matches = champions
      .filter(
        (c) =>
          c.name.toLowerCase().includes(q) ||
          c.alias.toLowerCase().includes(q),
      )
      .sort((a, b) => {
        const ap = a.name.toLowerCase().startsWith(q) ? 0 : 1;
        const bp = b.name.toLowerCase().startsWith(q) ? 0 : 1;
        return ap - bp || a.name.localeCompare(b.name);
      });
    for (const c of matches) results.push({ name: c.name, id: c.id });
  }

  const isPicked = (name: string) =>
    picked.some((n) => n.toLowerCase() === name.toLowerCase());

  const choose = (s: Suggestion) => {
    if (disabled || isPicked(s.name)) return;
    onAdd(s.name);
    setQuery("");
  };

  return (
    <div>
      <div className="relative">
        <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={query}
          disabled={disabled}
          placeholder={
            disabled ? `Priority list full (${MAX_PRIORITY})` : "Add a champion…"
          }
          className="h-9 pl-8"
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && results[0]) {
              e.preventDefault();
              choose(results[0]);
            }
          }}
        />
      </div>
      {results.length > 0 && (
        <div className="mt-1 max-h-56 overflow-y-auto rounded-md border border-edge bg-popover p-1">
          {results.map((s) => {
            const already = isPicked(s.name);
            return (
              <button
                key={s.name}
                type="button"
                disabled={already}
                onClick={() => choose(s)}
                className={cn(
                  "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent",
                  already && "opacity-40",
                )}
              >
                <ChampionIcon name={s.name} id={s.id} size={24} />
                <span className="flex-1 truncate">{s.name}</span>
                {already && (
                  <span className="text-xs text-muted-foreground">added</span>
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

function PriorityTiles({
  list,
  champions,
  role,
  verb,
  onRemove,
  onMove,
}: {
  list: string[];
  champions: ChampionInfo[];
  role: RoleKey;
  verb: string;
  onRemove: (i: number) => void;
  onMove: (from: number, to: number) => void;
}) {
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [overIndex, setOverIndex] = useState<number | null>(null);

  if (list.length === 0) {
    return (
      <p className="text-xs text-muted-foreground">
        {role === "default"
          ? `Empty - nothing will be ${verb}.`
          : "Empty - falls back to Default."}
      </p>
    );
  }

  const idOf = (name: string) => champions.find((c) => c.name === name)?.id;

  return (
    <div className="grid gap-2">
      <div className="flex flex-wrap gap-2">
        {list.map((name, i) => (
          <div
            key={name}
            draggable
            onDragStart={(e) => {
              setDragIndex(i);

              e.dataTransfer.effectAllowed = "move";
              e.dataTransfer.setData("text/plain", String(i));
            }}
            onDragOver={(e) => {
              e.preventDefault();
              e.dataTransfer.dropEffect = "move";
              setOverIndex(i);
            }}
            onDrop={(e) => {
              e.preventDefault();
              if (dragIndex !== null) onMove(dragIndex, i);
              setDragIndex(null);
              setOverIndex(null);
            }}
            onDragEnd={() => {
              setDragIndex(null);
              setOverIndex(null);
            }}
            title={name}
            className={cn(
              "group relative cursor-grab rounded-md transition active:cursor-grabbing",
              dragIndex === i && "opacity-40",
              overIndex === i &&
                dragIndex !== null &&
                dragIndex !== i &&
                "ring-2 ring-ring",
            )}
          >
            <ChampionIcon name={name} id={idOf(name)} size={44} />
            <span className="absolute -left-1.5 -top-1.5 flex size-4 items-center justify-center rounded-full bg-primary text-[10px] font-semibold text-primary-foreground ring-2 ring-panel">
              {i + 1}
            </span>
            <button
              type="button"
              onClick={() => onRemove(i)}
              title={`Remove ${name}`}
              className="absolute -bottom-1.5 -right-1.5 flex size-4 items-center justify-center rounded-full bg-destructive text-destructive-foreground opacity-0 ring-2 ring-panel transition group-hover:opacity-100"
            >
              <X className="size-2.5" strokeWidth={3} />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

function ChampionIcon({
  name,
  id,
  size = 40,
  className,
}: {
  name: string;
  id?: number;
  size?: number;
  className?: string;
}) {
  const [failed, setFailed] = useState(false);
  const box = { width: size, height: size };

  if (name === "Random") {
    return (
      <div
        style={box}
        title="Random"
        className={cn(
          "flex items-center justify-center rounded-md bg-panel2 text-muted-foreground",
          className,
        )}
      >
        <Dices className="size-1/2" />
      </div>
    );
  }

  if (id == null || failed) {
    return (
      <div
        style={box}
        title={name}
        className={cn(
          "flex items-center justify-center rounded-md bg-panel2 text-sm font-medium text-muted-foreground",
          className,
        )}
      >
        {name.charAt(0).toUpperCase()}
      </div>
    );
  }

  return (
    <img
      src={championIconUrl(id)}
      alt={name}
      title={name}
      loading="lazy"
      draggable={false}
      style={box}
      onError={() => setFailed(true)}
      className={cn("rounded-md object-cover", className)}
    />
  );
}

function RoleSpellPairs({
  value,
  onChange,
}: {
  value: RoleSpells;
  onChange: (next: RoleSpells) => void;
}) {
  const [role, setRole] = useState<RoleKey>("default");
  const [icons, setIcons] = useState<Map<number, string>>(new Map());
  const pair = value[role];

  const forceSmite = role === "jungle";
  const set = (slot: 0 | 1, spell: string) => {
    const other = slot === 0 ? 1 : 0;
    const next: [string, string] = [...pair];

    if (spell === next[other]) next[other] = next[slot];
    next[slot] = spell;z
    onChange({ ...value, [role]: next });
  };

  useEffect(() => {
    fetchSummonerSpellIcons().then(setIcons).catch(() => {});
  }, []);

  useEffect(() => {
    if (!forceSmite) return;
    if (pair[1] === "Smite" && pair[0] !== "Smite") return;
    const keep = pair[0] !== "Smite" ? pair[0] : pair[1] !== "Smite" ? pair[1] : "Flash";
    onChange({ ...value, jungle: [keep, "Smite"] });
  }, [forceSmite, pair, value, onChange]);

  const iconFor = (name: string) => icons.get(SPELL_ID[name] ?? -1);

  return (
    <div className="grid gap-2">
      <RoleTabs role={role} onSelect={setRole} marked={() => false} />
      <div className="flex items-center gap-2">
        {([0, 1] as const).map((slot) => {
          const locked = forceSmite && slot === 1;
          return (
            <label
              key={slot}
              className="flex flex-1 items-center gap-2 text-xs text-muted-foreground"
            >
              {slot === 0 ? "D" : "F"}
              <Select
                value={pair[slot]}
                disabled={locked}
                onValueChange={(v) => set(slot, v)}
              >
                <SelectTrigger size="sm" className="min-w-0 flex-1">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SUMMONER_SPELLS.map((s) => (
                    <SelectItem
                      key={s}
                      value={s}
                      disabled={forceSmite && slot === 0 && s === "Smite"}
                    >
                      <span className="flex items-center gap-2">
                        <SpellIcon name={s} url={iconFor(s)} />
                        {s}
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
          );
        })}
      </div>
      {forceSmite && (
        <p className="text-xs text-muted-foreground">
          Smite is locked for Jungle.
        </p>
      )}
    </div>
  );
}

function SpellIcon({
  name,
  url,
  size = 18,
}: {
  name: string;
  url?: string;
  size?: number;
}) {
  const [failed, setFailed] = useState(false);
  const box = { width: size, height: size };
  if (!url || failed) {
    return (
      <span
        style={box}
        title={name}
        className="inline-block shrink-0 rounded bg-panel2"
      />
    );
  }
  return (
    <img
      src={url}
      alt={name}
      title={name}
      loading="lazy"
      style={box}
      onError={() => setFailed(true)}
      className="shrink-0 rounded"
    />
  );
}
