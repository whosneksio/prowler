import { useState } from "react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { claimAllRewards, dodge, restartUx } from "../lib/ipc";
import { useLog } from "../lib/log";

export function GameToolsView() {
  const { log } = useLog();
  const [busy, setBusy] = useState<string | null>(null);

  async function run(key: string, action: () => Promise<unknown>) {
    setBusy(key);
    try {
      await action();
    } catch (e) {
      log(`${e}`);
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        title="Game Tools"
        subtitle="Lobby Reveal, Dodge, and Restart Client UX."
      />

      <div className="grid max-w-3xl gap-4">
        <Card
          title="Lobby Reveal"
          desc="Open your champ-select teammates in a multi-search. Provider is set in Settings."
          className="opacity-50"
        >
          <div className="flex items-center gap-3">
            <Button size="sm" disabled>
              Reveal lobby
            </Button>
          </div>
        </Card>

        <Card
          title="Dodge"
          desc="Quit the current champ select instantly, without waiting or closing the client."
        >
          <Button size="sm" variant="destructive" disabled={busy !== null} onClick={() => run("dodge", dodge)}>
            {busy === "dodge" ? "Dodging…" : "Dodge champ select"}
          </Button>
        </Card>

        <Card
          title="Claim All Rewards"
          desc="Claim event pass rewards, loot milestones, and pending level-up grants. Grants that need a manual pick are skipped."
        >
          <Button size="sm"
            disabled={busy !== null}
            onClick={() => run("claim", claimAllRewards)}
          >
            {busy === "claim" ? "Claiming…" : "Claim all rewards"}
          </Button>
        </Card>

        <Card
          title="Restart Client UX"
          desc="Restart the client's renderer without logging out - fixes a frozen or glitched client."
        >
          <Button size="sm"
            disabled={busy !== null}
            onClick={() => run("restart", restartUx)}
          >
            {busy === "restart" ? "Restarting…" : "Restart UX"}
          </Button>
        </Card>
      </div>
    </div>
  );
}
