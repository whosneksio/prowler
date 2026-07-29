import { useCallback, useEffect, useState } from "react";
import { ViewHeader } from "../components/common";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  deleteAccount,
  listAccounts,
  renameAccount,
  saveCurrentAccount,
  switchAccount,
} from "../lib/ipc";
import { useLog } from "../lib/log";
import { profileIconUrl } from "../lib/cdragon";
import type { AccountMeta, ConnectionStatus } from "../lib/types";

export function SwitcherView({
  status,
  showUsername = true,
}: {
  status: ConnectionStatus;
  showUsername?: boolean;
}) {
  const { log } = useLog();
  const [accounts, setAccounts] = useState<AccountMeta[]>([]);
  const [saving, setSaving] = useState(false);
  const [switchingId, setSwitchingId] = useState<string | null>(null);

  const refresh = useCallback(() => {
    listAccounts()
      .then(setAccounts)
      .catch((e) => log(`Failed to load accounts: ${e}`));
  }, [log]);

  useEffect(refresh, [refresh]);

  async function onSave() {
    setSaving(true);
    try {
      await saveCurrentAccount();
      refresh();
    } catch (e) {
      log(`Save failed: ${e}`);
    } finally {
      setSaving(false);
    }
  }

  async function onSwitch(account: AccountMeta) {
    setSwitchingId(account.id);
    try {
      await switchAccount(account.id);
    } catch (e) {
      log(`Switch failed: ${e}`);
    } finally {
      setSwitchingId(null);
    }
  }

  async function onRename(account: AccountMeta, label: string) {
    try {
      await renameAccount(account.id, label);
      refresh();
    } catch (e) {
      log(`Rename failed: ${e}`);
    }
  }

  async function onDelete(account: AccountMeta) {
    try {
      await deleteAccount(account.id);
      refresh();
    } catch (e) {
      log(`Delete failed: ${e}`);
    }
  }

  const currentPuuid = status.summoner?.puuid ?? null;

  return (
    <div className="flex h-full flex-col">
      <ViewHeader
        title="Accounts"
        subtitle="Save Riot sessions and switch between accounts in one click."
        action={
          <Button onClick={onSave} disabled={saving}>
            {saving ? "Saving…" : "Save current session"}
          </Button>
        }
      />

      {accounts.length === 0 ? (
        <div className="flex flex-1 items-center justify-center rounded-xl border border-dashed border-edge">
          <div className="max-w-md text-center">
            <p className="text-sm font-medium">No accounts saved yet</p>
            <p className="mt-2 text-sm text-muted-foreground">
              Log in to the Riot Client with{" "}
              <span className="text-text">“Stay signed in”</span> checked, then
              click <span className="text-text">Save current session</span>.
              Repeat for each account you want to switch between.
            </p>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
          {accounts.map((a) => (
            <AccountCard
              key={a.id}
              account={a}
              isCurrent={!!currentPuuid && a.puuid === currentPuuid}
              switching={switchingId === a.id}
              switchDisabled={switchingId !== null}
              blurIdentity={!showUsername}
              onSwitch={() => onSwitch(a)}
              onRename={(label) => onRename(a, label)}
              onDelete={() => onDelete(a)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function AccountCard({
  account,
  isCurrent,
  switching,
  switchDisabled,
  blurIdentity,
  onSwitch,
  onRename,
  onDelete,
}: {
  account: AccountMeta;
  isCurrent: boolean;
  switching: boolean;
  switchDisabled: boolean;
  blurIdentity: boolean;
  onSwitch: () => void;
  onRename: (label: string) => void;
  onDelete: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(account.label);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const riotId =
    account.gameName && account.tagLine
      ? `${account.gameName}#${account.tagLine}`
      : null;

  const blurClass =
    blurIdentity && !editing
      ? "blur-sm transition-[filter] duration-200 group-hover:blur-none"
      : "";

  function commitRename() {
    setEditing(false);
    const label = draft.trim();
    if (label && label !== account.label) onRename(label);
    else setDraft(account.label);
  }

  return (
    <div
      className={`group rounded-xl border bg-panel p-4 ${
        isCurrent ? "border-primary/25" : "border-edge"
      }`}
    >
      <div className="flex items-center gap-3">
        <AccountIcon account={account} />
        <div className="min-w-0 flex-1">
          {editing ? (
            <Input
              autoFocus
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              onBlur={commitRename}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitRename();
                if (e.key === "Escape") {
                  setDraft(account.label);
                  setEditing(false);
                }
              }}
              className="h-7 px-2"
            />
          ) : (
            <p className={`truncate text-sm font-semibold ${blurClass}`}>
              {account.label}
            </p>
          )}
          <p className={`mt-0.5 truncate text-xs text-muted-foreground ${blurClass}`}>
            {riotId ?? "Riot ID unknown"}
            {account.region ? ` · ${account.region.toUpperCase()}` : ""}
          </p>
        </div>
        {isCurrent && (
          <Badge className="rounded-full bg-primary/20 text-[11px] text-foreground">
            Active
          </Badge>
        )}
      </div>

      <div className="mt-4 flex items-center gap-2">
        <Button
          size="sm"
          className="flex-1"
          onClick={onSwitch}
          disabled={switchDisabled || isCurrent}
        >
          {switching ? "Switching…" : isCurrent ? "Signed in" : "Switch"}
        </Button>
        <Button
          size="sm"
          variant="secondary"
          onClick={() => {
            setDraft(account.label);
            setEditing(true);
          }}
        >
          Rename
        </Button>
        {confirmDelete ? (
          <Button
            size="sm"
            variant="destructive"
            onClick={onDelete}
            onBlur={() => setConfirmDelete(false)}
          >
            Confirm
          </Button>
        ) : (
          <Button
            size="sm"
            variant="secondary"
            className="hover:text-destructive"
            onClick={() => setConfirmDelete(true)}
          >
            Delete
          </Button>
        )}
      </div>
    </div>
  );
}

function AccountIcon({ account }: { account: AccountMeta }) {
  const [failed, setFailed] = useState(false);
  const initial = (account.label || "?").slice(0, 1).toUpperCase();

  if (!account.profileIconId || failed) {
    return (
      <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-panel2 text-sm font-semibold text-muted-foreground">
        {initial}
      </div>
    );
  }
  return (
    <img
      src={profileIconUrl(account.profileIconId)}
      onError={() => setFailed(true)}
      alt=""
      className="h-11 w-11 shrink-0 rounded-full border border-edge object-cover"
    />
  );
}
