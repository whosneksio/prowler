import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TitleBar } from "./components/TitleBar";
import { LogFeed } from "./components/LogFeed";
import { LogProvider, useLog } from "./lib/log";
import { getConfig, getConnectionStatus, onConfig, onStatus } from "./lib/ipc";
import type { Config, ConnectionStatus, ViewId } from "./lib/types";
import { SwitcherView } from "./views/SwitcherView";
import { AutomationView } from "./views/AutomationView";
import { CustomizationView } from "./views/CustomizationView";
import { RunesView } from "./views/RunesView";
import { GameToolsView } from "./views/GameToolsView";
import { SocialView } from "./views/SocialView";
import { SettingsView } from "./views/SettingsView";

function Shell() {
  const { log } = useLog();
  const [view, setView] = useState<ViewId>("switcher");
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    summoner: null,
    phase: null,
    region: null,
  });
  const [showUsername, setShowUsername] = useState(true);

  useEffect(() => {
    let prevConnected: boolean | null = null;
    getConnectionStatus().then(setStatus).catch(() => {});
    getConfig()
      .then((c) => setShowUsername(c.ui.show_username))
      .catch(() => {});
    const un = onStatus((s) => {
      setStatus(s);
      if (prevConnected !== s.connected) {
        if (s.connected) log("League client connected.", "success");
        else log("League client disconnected.", "warn");
        prevConnected = s.connected;
      }
    });
    const unConfig = onConfig((c: Config) => setShowUsername(c.ui.show_username));
    return () => {
      un.then((f) => f());
      unConfig.then((f) => f());
    };
  }, [log]);

  const gated = !status.connected && view !== "switcher" && view !== "settings";

  return (
    <div className="flex h-screen flex-col">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar active={view} onSelect={setView} />
        <div className="flex min-w-0 flex-1 flex-col">
          <StatusBar status={status} showUsername={showUsername} />
          <main className="relative min-h-0 min-w-0 flex-1 overflow-hidden">
            <div
              className={cn(
                "h-full overflow-y-auto p-6",
                gated && "pointer-events-none select-none blur-[3px]",
              )}
              aria-hidden={gated}
            >
              {view === "switcher" && (
                <SwitcherView status={status} showUsername={showUsername} />
              )}
              {view === "automation" && <AutomationView />}
              {view === "customization" && <CustomizationView />}
              {view === "runes" && <RunesView />}
              {view === "tools" && <GameToolsView />}
              {view === "social" && <SocialView />}
              {view === "settings" && <SettingsView />}
            </div>
            {gated && <ClientGate />}
          </main>
        </div>
      </div>
      <LogFeed />
    </div>
  );
}

function ClientGate() {
  return (
    <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-background/50 text-center">
      <div>
        <p className="text-sm font-medium">Connect to your League client first</p>
        <p className="mt-1 text-xs text-muted-foreground">
          Start League and sign in - this tab needs a live client connection.
        </p>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <LogProvider>
      <Shell />
    </LogProvider>
  );
}
