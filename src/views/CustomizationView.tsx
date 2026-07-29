import { useEffect, useMemo, useState } from "react";
import { Search } from "lucide-react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { fetchSkins, profileIconUrl, type SkinInfo } from "../lib/cdragon";
import {
  setBackground,
  setBadges,
  setClientIcon,
  setStatusMessage,
  type BadgeMode,
} from "../lib/ipc";
import { useLog } from "../lib/log";

export function CustomizationView() {
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        title="Customize"
        subtitle="Profile icon, background, badges, and status message."
      />
      <div className="grid max-w-3xl gap-4">
        <div className="grid gap-4 md:grid-cols-2">
          <IconCard />
          <BadgesCard />
        </div>
        <StatusCard />
        <BackgroundCard />
      </div>
    </div>
  );
}

function useAction() {
  const { log } = useLog();
  const [busy, setBusy] = useState(false);
  async function run(action: () => Promise<unknown>) {
    setBusy(true);
    try {
      await action();
    } catch (e) {
      log(`${e}`);
    } finally {
      setBusy(false);
    }
  }
  return { busy, run };
}

function IconCard() {
  const { busy, run } = useAction();
  const [iconId, setIconId] = useState("");
  const id = Number(iconId);
  const valid = iconId !== "" && Number.isInteger(id) && id >= 0;

  return (
    <Card
      title="Icons"
      desc="Client-sided only chat icon."
      contentClassName="flex flex-1 flex-col justify-end"
    >
      <div className="flex items-center gap-2">
        {valid ? (
          <img
            src={profileIconUrl(id)}
            alt=""
            className="h-10 w-10 rounded-full border border-edge object-cover"
          />
        ) : (
          <div className="h-10 w-10 rounded-full border border-dashed border-edge" />
        )}
        <Input
          placeholder="Icon id (e.g. 29)"
          value={iconId}
          onChange={(e) => setIconId(e.target.value.replace(/\D/g, ""))}
          className="w-40"
        />
        <Button size="sm"
          disabled={busy || !valid}
          onClick={() => run(() => setClientIcon(id))}
        >
          Set temporarily
        </Button>
      </div>
    </Card>
  );
}

function StatusCard() {
  const { busy, run } = useAction();
  const [message, setMessage] = useState("");

  return (
    <Card title="Status message" desc="Shown under your name in friends lists. Multi-line and ASCII art work.">
      <div className="grid gap-2">
        <Textarea
          placeholder="Your status…"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          rows={8}
          wrap="off"
          className="whitespace-pre overflow-auto font-mono text-xs leading-tight"
        />
        <div className="flex gap-2">
          <Button size="sm"
            disabled={busy}
            onClick={() => run(() => setStatusMessage(message))}
          >
            Set
          </Button>
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => run(() => setStatusMessage(""))}>
            Clear
          </Button>
        </div>
      </div>
    </Card>
  );
}

function BadgesCard() {
  const { busy, run } = useAction();
  const apply = (mode: BadgeMode) => run(() => setBadges(mode));

  return (
    <Card
      title="Profile badges"
      desc="Rearrange the challenge badges on your profile."
      contentClassName="flex flex-1 flex-col justify-end"
    >
      <div className="flex gap-2">
        <Button size="sm" disabled={busy} onClick={() => apply("glitch")}>
          Glitch
        </Button>
        <Button size="sm" disabled={busy} onClick={() => apply("clear")}>
          Clear
        </Button>
      </div>
    </Card>
  );
}

const PAGE_SIZE = 24;

function BackgroundCard() {
  const { log } = useLog();
  const { busy, run } = useAction();
  const [skins, setSkins] = useState<SkinInfo[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(PAGE_SIZE);

  async function load() {
    setLoading(true);
    try {
      setSkins(await fetchSkins());
    } catch (e) {
      log(`Failed to load skins from CDragon: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    setLimit(PAGE_SIZE);
  }, [query]);

  const filtered = useMemo(() => {
    if (!skins) return [];
    const q = query.trim().toLowerCase();
    return q ? skins.filter((s) => s.name.toLowerCase().includes(q)) : skins;
  }, [skins, query]);

  return (
    <Card
      title="Profile background"
      desc="Set any champion skin splash as your profile backdrop - no ownership required."
    >
      {!skins ? (
        <Button size="sm" disabled={loading} onClick={load}>
          {loading ? "Loading skins…" : "Browse skins"}
        </Button>
      ) : (
        <div>
          <div className="relative mb-3">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search skins…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="w-full pl-8"
            />
          </div>
          <div className="grid grid-cols-4 gap-2.5 md:grid-cols-6">
            {filtered.slice(0, limit).map((s) => (
              <button
                key={s.id}
                title={s.name}
                disabled={busy}
                onClick={() => run(() => setBackground(s.id))}
                className="group relative aspect-square overflow-hidden rounded-lg border border-edge outline-none transition duration-200 hover:border-primary/70 focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
              >
                <img
                  src={s.tileUrl}
                  alt={s.name}
                  loading="lazy"
                  className="h-full w-full object-cover transition-transform duration-300 ease-out group-hover:scale-110"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/15 to-transparent" />
                <span className="absolute inset-x-0 bottom-0 truncate px-2 pb-1.5 pt-5 text-left text-[11px] font-medium text-white/95">
                  {s.name}
                </span>
              </button>
            ))}
          </div>
          {filtered.length > limit && (
            <div className="mt-3 flex justify-center">
              <Button size="sm" variant="secondary" onClick={() => setLimit((l) => l + PAGE_SIZE * 2)}>
                Show more ({filtered.length - limit} left)
              </Button>
            </div>
          )}
          {filtered.length === 0 && (
            <p className="py-4 text-center text-sm text-muted-foreground">
              No skins match “{query}”.
            </p>
          )}
        </div>
      )}
    </Card>
  );
}
