import { useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TitleBar } from "./components/TitleBar";
import { LogFeed } from "./components/LogFeed";
import { LogProvider, useLog } from "./lib/log";
import { getConnectionStatus, onStatus } from "./lib/ipc";
import type { ConnectionStatus, ViewId } from "./lib/types";
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
  });

  useEffect(() => {
    let prevConnected: boolean | null = null;
    getConnectionStatus().then(setStatus).catch(() => {});
    const un = onStatus((s) => {
      setStatus(s);
      if (prevConnected !== s.connected) {
        if (s.connected) log("League client connected.", "success");
        else log("League client disconnected.", "warn");
        prevConnected = s.connected;
      }
    });
    return () => {
      un.then((f) => f());
    };
  }, [log]);

  return (
    <div className="flex h-screen flex-col">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar active={view} onSelect={setView} />
        <div className="flex min-w-0 flex-1 flex-col">
          <StatusBar status={status} />
          <main className="min-h-0 min-w-0 flex-1 overflow-y-auto p-6">
            {view === "switcher" && <SwitcherView status={status} />}
            {view === "automation" && <AutomationView />}
            {view === "customization" && <CustomizationView />}
            {view === "runes" && <RunesView />}
            {view === "tools" && <GameToolsView />}
            {view === "social" && <SocialView />}
            {view === "settings" && <SettingsView />}
          </main>
        </div>
      </div>
      <LogFeed />
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
