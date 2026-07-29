import type { ConnectionStatus } from "../lib/types";
import { Badge } from "@/components/ui/badge";

export function StatusBar({ status }: { status: ConnectionStatus }) {
  const connected = status.connected;
  const summoner = status.summoner;

  return (
    <header className="flex items-center justify-between border-b border-edge bg-panel px-5 py-3">
      <div className="flex items-center gap-3">
        <span
          className={`inline-block h-2.5 w-2.5 rounded-full ${
            connected ? "bg-good" : "bg-bad"
          }`}
        />
        <span className="text-sm font-medium">
          {connected ? "Client connected" : "Client not detected"}
        </span>
        {connected && summoner && (
          <span className="text-sm text-muted-foreground">
            · {summoner.gameName ? `${summoner.gameName}#${summoner.tagLine}` : summoner.displayName}
            {summoner.summonerLevel ? ` · Level ${summoner.summonerLevel}` : ""}
          </span>
        )}
      </div>
      {connected && status.phase && (
        <Badge variant="secondary" className="rounded-full text-muted-foreground">
          {status.phase}
        </Badge>
      )}
    </header>
  );
}
