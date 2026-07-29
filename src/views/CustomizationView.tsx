import { useEffect, useMemo, useState } from "react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
        <IconCard />
        <StatusCard />
        <BadgesCard />
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
          Set client-side only
        </Button>
      </div>
    </Card>
  );
}

function StatusCard() {
  const { busy, run } = useAction();
  const [message, setMessage] = useState("");

  return (
    <Card title="Status message" desc="Shown under your name in friends lists.">
      <div className="flex items-center gap-2">
        <Input
          placeholder="Your status…"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          className="flex-1"
        />
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
          <Input
            placeholder="Search skins…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="mb-3 w-full"
          />
          <div className="grid grid-cols-4 gap-2 md:grid-cols-6">
            {filtered.slice(0, limit).map((s) => (
              <button
                key={s.id}
                title={s.name}
                disabled={busy}
                onClick={() => run(() => setBackground(s.id))}
                className="group overflow-hidden rounded-lg border border-edge transition-colors hover:border-primary disabled:opacity-50"
              >
                <img
                  src={s.tileUrl}
                  alt={s.name}
                  loading="lazy"
                  className="aspect-square w-full object-cover"
                />
                <span className="block truncate px-1.5 py-1 text-[11px] text-muted-foreground group-hover:text-text">
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
