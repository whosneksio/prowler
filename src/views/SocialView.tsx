import { useEffect, useRef, useState } from "react";
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
  const [confirming, setConfirming] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timer.current) clearTimeout(timer.current);
    };
  }, []);

  function arm() {
    setConfirming(true);
    if (timer.current) clearTimeout(timer.current);

    timer.current = setTimeout(() => setConfirming(false), 2000);
  }

  async function confirmRemove() {
    if (timer.current) clearTimeout(timer.current);
    setConfirming(false);
    setBusy(true);
    try {
      const n = await countFriends();
      if (n === 0) {
        log("Friends list is already empty.");
      } else {
        await removeAllFriends();
        log(`Removed ${n} friends.`, "success");
      }
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
      <Button
        size="sm"
        variant="destructive"
        disabled={busy}
        onClick={confirming ? confirmRemove : arm}
      >
        {busy ? "Working…" : confirming ? "Click again to confirm" : "Remove all friends"}
      </Button>
    </Card>
  );
}
