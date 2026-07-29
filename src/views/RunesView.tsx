import { useEffect, useRef, useState } from "react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  applyRunePage,
  getConfig,
  getRuneTrees,
  onStatus,
  setConfig,
} from "../lib/ipc";
import { useLog } from "../lib/log";
import type { Config, Perk, RuneData, RunePage, RuneTree } from "../lib/types";

const TREE_ACCENT: Record<number, string> = {
  8000: "#C8AA6E",
  8100: "#D44A4A",
  8200: "#3D9BE0",
  8300: "#49B6C4",
  8400: "#4CAF7A",
};
const accentOf = (id: number | null | undefined) =>
  (id != null && TREE_ACCENT[id]) || "#C8AA6E";

interface Draft {
  index: number | null;
  name: string;
  primaryStyleId: number | null;
  subStyleId: number | null;
  keystone: number | null;
  primary: (number | null)[];
  secondary: { row: number; perk: number }[];
  shards: (number | null)[];
}

function keystoneSlot(tree: RuneTree) {
  return tree.slots.find((s) => /keystone/i.test(s.type)) ?? tree.slots[0];
}

function minorSlots(tree: RuneTree) {
  const key = keystoneSlot(tree);
  return tree.slots.filter((s) => s !== key);
}

function emptyDraft(): Draft {
  return {
    index: null,
    name: "",
    primaryStyleId: null,
    subStyleId: null,
    keystone: null,
    primary: [],
    secondary: [],
    shards: [],
  };
}

function draftFromPage(page: RunePage, index: number, data: RuneData): Draft {
  const primaryTree = data.trees.find((t) => t.id === page.primary_style_id);
  const ids = new Set(page.selected_perk_ids);

  let keystone: number | null = null;
  const primary: (number | null)[] = [];
  const secondary: { row: number; perk: number }[] = [];

  if (primaryTree) {
    const ks = keystoneSlot(primaryTree);
    keystone = ks.perks.find((p) => ids.has(p)) ?? null;
    for (const slot of minorSlots(primaryTree)) {
      primary.push(slot.perks.find((p) => ids.has(p)) ?? null);
    }
  }
  const secondaryTree = data.trees.find((t) => t.id === page.sub_style_id);
  if (secondaryTree) {
    minorSlots(secondaryTree).forEach((slot, row) => {
      const perk = slot.perks.find((p) => ids.has(p));
      if (perk != null) secondary.push({ row, perk });
    });
  }
  const shards = data.shards.map((row) => row.find((p) => ids.has(p)) ?? null);

  return {
    index,
    name: page.name,
    primaryStyleId: page.primary_style_id,
    subStyleId: page.sub_style_id,
    keystone,
    primary,
    secondary,
    shards,
  };
}

function serializeDraft(draft: Draft): RunePage {
  const perks: number[] = [];
  if (draft.keystone != null) perks.push(draft.keystone);
  for (const p of draft.primary) if (p != null) perks.push(p);
  for (const s of draft.secondary) perks.push(s.perk);
  for (const s of draft.shards) if (s != null) perks.push(s);
  return {
    name: draft.name.trim() || "Custom Page",
    primary_style_id: draft.primaryStyleId ?? 0,
    sub_style_id: draft.subStyleId ?? 0,
    selected_perk_ids: perks,
  };
}

function draftComplete(draft: Draft, data: RuneData): boolean {
  const primaryTree = data.trees.find((t) => t.id === draft.primaryStyleId);
  const secondaryTree = data.trees.find((t) => t.id === draft.subStyleId);
  if (!primaryTree || !secondaryTree) return false;
  if (draft.primaryStyleId === draft.subStyleId) return false;
  if (draft.keystone == null) return false;
  const minors = minorSlots(primaryTree).length;
  if (draft.primary.length !== minors || draft.primary.some((p) => p == null)) {
    return false;
  }
  if (draft.secondary.length !== 2) return false;
  if (data.shards.length > 0 && draft.shards.some((s) => s == null)) return false;
  if (draft.shards.length !== data.shards.length) return false;
  return true;
}

export function RunesView() {
  const { log } = useLog();
  const [config, setLocal] = useState<Config | null>(null);
  const [data, setData] = useState<RuneData | null>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [busy, setBusy] = useState<number | null>(null);
  const dataLoaded = useRef(false);

  useEffect(() => {
    getConfig().then(setLocal).catch((e) => log(`Failed to load config: ${e}`));
  }, [log]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const load = () => {
      if (dataLoaded.current) return;
      getRuneTrees()
        .then((d) => {
          dataLoaded.current = true;
          setData(d);
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
        <ViewHeader title="Runes" />
        <p className="text-sm text-muted-foreground">Loading…</p>
      </div>
    );
  }

  const pages = config.custom_runes.pages;

  async function savePages(next: RunePage[]) {
    const nextConfig = { ...config!, custom_runes: { pages: next } };
    setLocal(nextConfig);
    try {
      await setConfig(nextConfig);
    } catch (e) {
      log(`Failed to save rune pages: ${e}`);
    }
  }

  async function apply(page: RunePage, i: number) {
    setBusy(i);
    try {
      await applyRunePage(page);
      log(`Applied rune page "${page.name}".`, "success");
    } catch (e) {
      log(`Failed to apply "${page.name}": ${e}`);
    } finally {
      setBusy(null);
    }
  }

  function commitDraft(d: Draft) {
    const page = serializeDraft(d);
    const next =
      d.index == null
        ? [...pages, page]
        : pages.map((p, i) => (i === d.index ? page : p));
    void savePages(next);
    setDraft(null);
  }

  if (draft && data) {
    return (
      <RuneBuilder
        draft={draft}
        data={data}
        onChange={setDraft}
        onSave={commitDraft}
        onCancel={() => setDraft(null)}
      />
    );
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        title="Runes"
        subtitle="Build your own rune pages and apply them to the client on demand."
        action={
          <Button
            size="sm"
            disabled={!data}
            onClick={() => setDraft(emptyDraft())}
          >
            + New page
          </Button>
        }
      />

      {!data && (
        <p className="mb-4 text-sm text-muted-foreground">
          Start League once to load the rune trees, then create a page.
        </p>
      )}

      <div className="grid max-w-xl gap-3">
        {pages.length === 0 && (
          <Card>
            <p className="text-sm text-muted-foreground">
              No rune pages yet. Click “New page” to build one.
            </p>
          </Card>
        )}
        {pages.map((page, i) => (
          <PageRow
            key={i}
            page={page}
            data={data}
            busy={busy === i}
            onApply={() => apply(page, i)}
            onEdit={
              data ? () => setDraft(draftFromPage(page, i, data)) : undefined
            }
            onDelete={() => savePages(pages.filter((_, j) => j !== i))}
          />
        ))}
      </div>
    </div>
  );
}

function PageRow({
  page,
  data,
  busy,
  onApply,
  onEdit,
  onDelete,
}: {
  page: RunePage;
  data: RuneData | null;
  busy: boolean;
  onApply: () => void;
  onEdit?: () => void;
  onDelete: () => void;
}) {
  const primary = data?.trees.find((t) => t.id === page.primary_style_id);
  const secondary = data?.trees.find((t) => t.id === page.sub_style_id);
  const keystoneId = primary
    ? keystoneSlot(primary).perks.find((id) =>
        page.selected_perk_ids.includes(id),
      )
    : undefined;
  const keystone =
    keystoneId != null ? data?.perks[String(keystoneId)] : undefined;
  return (
    <Card className="p-3">
      <div className="flex items-center gap-3">
        <div className="flex shrink-0 items-center gap-1.5">
          {keystone?.icon ? (
            <img
              src={keystone.icon}
              alt={keystone.name}
              className="size-8 rounded-full"
            />
          ) : (
            primary?.icon && (
              <img src={primary.icon} alt={primary.name} className="size-7" />
            )
          )}
          {secondary?.icon && (
            <img
              src={secondary.icon}
              alt={secondary.name}
              className="size-4 opacity-70"
            />
          )}
        </div>
        <p className="min-w-0 flex-1 truncate text-sm text-text">{page.name}</p>
        <Button size="sm" disabled={busy} onClick={onApply}>
          {busy ? "Applying…" : "Apply"}
        </Button>
        {onEdit && (
          <Button size="sm" variant="secondary" onClick={onEdit}>
            Edit
          </Button>
        )}
        <Button
          size="sm"
          variant="ghost"
          className="h-8 px-2 text-muted-foreground"
          onClick={onDelete}
        >
          ✕
        </Button>
      </div>
    </Card>
  );
}

function PerkIcon({
  perk,
  selected,
  onClick,
  accent = "#C8AA6E",
  size = "size-12",
}: {
  perk: Perk | undefined;
  selected: boolean;
  onClick: () => void;
  accent?: string;
  size?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={perk?.name}
      style={
        selected
          ? { boxShadow: `0 0 0 2px ${accent}, 0 0 14px -2px ${accent}` }
          : undefined
      }
      className={`relative flex ${size} shrink-0 items-center justify-center overflow-hidden rounded-full border transition motion-safe:hover:scale-105 ${
        selected
          ? "border-transparent motion-safe:scale-105"
          : "border-edge bg-panel2 opacity-45 grayscale-[0.4] hover:opacity-100 hover:grayscale-0"
      }`}
    >
      {perk?.icon ? (
        <img
          src={perk.icon}
          alt={perk?.name}
          className="size-full rounded-full"
        />
      ) : (
        <span className="text-[10px] text-muted-foreground">{perk?.id}</span>
      )}
    </button>
  );
}

function TreePicker({
  trees,
  value,
  exclude,
  onSelect,
}: {
  trees: RuneTree[];
  value: number | null;
  exclude?: number | null;
  onSelect: (id: number) => void;
}) {
  return (
    <div className="flex flex-wrap gap-4">
      {trees.map((tree) => {
        const disabled = exclude != null && tree.id === exclude;
        const active = value === tree.id;
        const accent = accentOf(tree.id);
        return (
          <button
            key={tree.id}
            type="button"
            disabled={disabled}
            onClick={() => onSelect(tree.id)}
            title={tree.name}
            className={`group flex flex-col items-center gap-1.5 ${
              disabled ? "cursor-not-allowed opacity-30" : ""
            }`}
          >
            <span
              style={
                active
                  ? { boxShadow: `0 0 0 2px ${accent}, 0 0 12px -2px ${accent}` }
                  : undefined
              }
              className={`flex size-14 items-center justify-center rounded-full border transition motion-safe:group-hover:scale-105 ${
                active
                  ? "border-transparent"
                  : "border-edge bg-panel2 opacity-50 grayscale-[0.4] group-hover:opacity-100 group-hover:grayscale-0"
              }`}
            >
              {tree.icon && <img src={tree.icon} alt="" className="size-9" />}
            </span>
            <span
              className={`text-xs transition-colors ${
                active ? "text-text" : "text-muted-foreground"
              }`}
            >
              {tree.name}
            </span>
          </button>
        );
      })}
    </div>
  );
}

function RuneBuilder({
  draft,
  data,
  onChange,
  onSave,
  onCancel,
}: {
  draft: Draft;
  data: RuneData;
  onChange: (d: Draft) => void;
  onSave: (d: Draft) => void;
  onCancel: () => void;
}) {
  const perk = (id: number | null | undefined): Perk | undefined =>
    id == null ? undefined : data.perks[String(id)];

  const primaryTree = data.trees.find((t) => t.id === draft.primaryStyleId);
  const secondaryTree = data.trees.find((t) => t.id === draft.subStyleId);

  function pickPrimaryTree(id: number) {
    const tree = data.trees.find((t) => t.id === id)!;
    onChange({
      ...draft,
      primaryStyleId: id,
      keystone: null,
      primary: minorSlots(tree).map(() => null),
      subStyleId: draft.subStyleId === id ? null : draft.subStyleId,
      secondary: draft.subStyleId === id ? [] : draft.secondary,
    });
  }

  function pickSecondaryTree(id: number) {
    onChange({ ...draft, subStyleId: id, secondary: [] });
  }

  function setPrimaryRow(row: number, perkId: number) {
    const primary = [...draft.primary];
    primary[row] = perkId;
    onChange({ ...draft, primary });
  }

  function toggleSecondary(row: number, perkId: number) {
    const existing = draft.secondary.find((s) => s.row === row);
    let secondary: { row: number; perk: number }[];
    if (existing?.perk === perkId) {
      secondary = draft.secondary.filter((s) => s.row !== row);
    } else if (existing) {
      secondary = draft.secondary.map((s) =>
        s.row === row ? { row, perk: perkId } : s,
      );
    } else if (draft.secondary.length < 2) {
      secondary = [...draft.secondary, { row, perk: perkId }];
    } else {
      return;
    }
    onChange({ ...draft, secondary });
  }

  function setShard(row: number, perkId: number) {
    const shards =
      draft.shards.length === data.shards.length
        ? [...draft.shards]
        : data.shards.map(() => null);
    shards[row] = perkId;
    onChange({ ...draft, shards });
  }

  const complete = draftComplete(draft, data);

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        title={draft.index == null ? "New rune page" : "Edit rune page"}
        action={
          <div className="flex gap-2">
            <Button size="sm" variant="ghost" onClick={onCancel}>
              Cancel
            </Button>
            <Button size="sm" disabled={!complete} onClick={() => onSave(draft)}>
              Save
            </Button>
          </div>
        }
      />

      <div className="grid max-w-5xl gap-4 md:grid-cols-2">
        <Card title="Name" className="md:col-span-2">
          <Input
            placeholder="e.g. Lethality Jhin"
            value={draft.name}
            onChange={(e) => onChange({ ...draft, name: e.target.value })}
            className="max-w-xs"
          />
        </Card>

        <Card title="Primary tree" desc="Pick a keystone and one rune per row.">
          <div className="grid gap-4">
            <TreePicker
              trees={data.trees}
              value={draft.primaryStyleId}
              onSelect={pickPrimaryTree}
            />
            {primaryTree && (
              <div className="grid gap-3">
                <p className="text-center text-[11px] font-semibold uppercase tracking-widest text-muted-foreground">
                  Keystones
                </p>
                <RuneRow
                  perks={keystoneSlot(primaryTree).perks}
                  perk={perk}
                  accent={accentOf(draft.primaryStyleId)}
                  variant="keystone"
                  isSelected={(id) => draft.keystone === id}
                  onPick={(id) => onChange({ ...draft, keystone: id })}
                />
                <hr className="my-1 border-edge" />
                {minorSlots(primaryTree).map((slot, row) => (
                  <RuneRow
                    key={row}
                    perks={slot.perks}
                    perk={perk}
                    accent={accentOf(draft.primaryStyleId)}
                    isSelected={(id) => draft.primary[row] === id}
                    onPick={(id) => setPrimaryRow(row, id)}
                  />
                ))}
              </div>
            )}
          </div>
        </Card>

        <Card
          title="Secondary tree"
          desc="Pick two runes from different rows."
        >
          <div className="grid gap-4">
            <TreePicker
              trees={data.trees}
              value={draft.subStyleId}
              exclude={draft.primaryStyleId}
              onSelect={pickSecondaryTree}
            />
            {secondaryTree && (
              <div className="grid gap-3">
                {minorSlots(secondaryTree).map((slot, row) => (
                  <RuneRow
                    key={row}
                    perks={slot.perks}
                    perk={perk}
                    accent={accentOf(draft.subStyleId)}
                    isSelected={(id) =>
                      draft.secondary.some(
                        (s) => s.row === row && s.perk === id,
                      )
                    }
                    onPick={(id) => toggleSecondary(row, id)}
                  />
                ))}
              </div>
            )}
          </div>
        </Card>

        {data.shards.length > 0 && (
          <Card title="Stat shards" desc="Pick one per row." className="md:col-span-2">
            <div className="grid gap-3">
              {data.shards.map((rowPerks, row) => (
                <RuneRow
                  key={row}
                  perks={rowPerks}
                  perk={perk}
                  variant="shard"
                  isSelected={(id) => draft.shards[row] === id}
                  onPick={(id) => setShard(row, id)}
                />
              ))}
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

function RuneRow({
  perks,
  perk,
  isSelected,
  onPick,
  accent,
  variant = "minor",
}: {
  perks: number[];
  perk: (id: number | null | undefined) => Perk | undefined;
  isSelected: (id: number) => boolean;
  onPick: (id: number) => void;
  accent?: string;
  variant?: "keystone" | "minor" | "shard";
}) {
  const size =
    variant === "keystone"
      ? "size-16"
      : variant === "shard"
        ? "size-8"
        : "size-12";
  const gap =
    variant === "keystone" ? "gap-4" : variant === "shard" ? "gap-2" : "gap-3";
  return (
    <div className={`flex flex-wrap items-center justify-center ${gap}`}>
      {perks.map((id) => (
        <PerkIcon
          key={id}
          perk={perk(id)}
          selected={isSelected(id)}
          onClick={() => onPick(id)}
          accent={accent}
          size={size}
        />
      ))}
    </div>
  );
}
