import { useState } from "react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { countFriends, removeAllFriends, setChatOffline } from "../lib/ipc";
import { useLog } from "../lib/log";

export function SocialView() {
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        title="Social"
        subtitle="Toggle chat offline and manage your friends list."
      />
      <div className="grid max-w-3xl gap-4">
        <OfflineCard />
        <RemoveFriendsCard />
      </div>
    </div>
  );
}

function OfflineCard() {
  const { log } = useLog();
  const [busy, setBusy] = useState(false);

  async function toggle(offline: boolean) {
    setBusy(true);
    try {
      await setChatOffline(offline);
    } catch (e) {
      log(`${e}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card
      title="Appear offline"
      desc="Disconnect from chat so friends see you offline while you keep playing. Reconnect to appear online again."
    >
      <div className="flex gap-2">
        <Button size="sm" disabled={busy} onClick={() => toggle(true)}>
          Go offline
        </Button>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => toggle(false)}>
          Go online
        </Button>
      </div>
    </Card>
  );
}

function RemoveFriendsCard() {
  const { log } = useLog();
  const [busy, setBusy] = useState(false);
  const [pendingCount, setPendingCount] = useState<number | null>(null);

  async function askConfirm() {
    setBusy(true);
    try {
      const n = await countFriends();
      if (n === 0) {
        log("Friends list is already empty.");
      } else {
        setPendingCount(n);
      }
    } catch (e) {
      log(`${e}`);
    } finally {
      setBusy(false);
    }
  }

  async function confirmRemove() {
    setBusy(true);
    setPendingCount(null);
    try {
      await removeAllFriends();
    } catch (e) {
      log(`${e}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card
      title="Remove all friends"
      desc="Deletes every friend from your list. This cannot be undone."
    >
      {pendingCount === null ? (
        <Button size="sm" variant="destructive" disabled={busy} onClick={askConfirm}>
          {busy ? "Working…" : "Remove all friends"}
        </Button>
      ) : (
        <div className="flex items-center gap-3">
          <span className="text-sm">
            Delete <span className="font-semibold text-bad">{pendingCount}</span>{" "}
            friends permanently?
          </span>
          <Button size="sm" variant="destructive" disabled={busy} onClick={confirmRemove}>
            Yes, delete all
          </Button>
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => setPendingCount(null)}>
            Cancel
          </Button>
        </div>
      )}
    </Card>
  );
}
