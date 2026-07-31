import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TitleBar } from "./components/TitleBar";
import { LogFeed } from "./components/LogFeed";
import { UpdateBanner } from "./components/UpdateBanner";
import { LogProvider, useLog } from "./lib/log";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  getConfig,
  getConnectionStatus,
  installUpdate,
  onCloseRequested,
  onConfig,
  onStatus,
  onUpdate,
  onUpdateProgress,
  quit,
  setConfig,
} from "./lib/ipc";
import type {
  Config,
  ConnectionStatus,
  UpdateInfo,
  UpdateProgress,
  ViewId,
} from "./lib/types";
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
  const [config, setConfigLocal] = useState<Config | null>(null);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [dismissed, setDismissed] = useState<string | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [installing, setInstalling] = useState(false);
  const [closePrompt, setClosePrompt] = useState(false);

  useEffect(() => {
    let prevConnected: boolean | null = null;
    getConnectionStatus().then(setStatus).catch(() => {});
    getConfig().then(setConfigLocal).catch(() => {});
    const un = onStatus((s) => {
      setStatus(s);
      if (prevConnected !== s.connected) {
        if (s.connected) log("League client connected.", "success");
        else log("League client disconnected.", "warn");
        prevConnected = s.connected;
      }
    });
    const unConfig = onConfig(setConfigLocal);
    const unUpdate = onUpdate(setUpdate);
    const unProgress = onUpdateProgress(setProgress);
    const unClose = onCloseRequested(() => setClosePrompt(true));
    return () => {
      un.then((f) => f());
      unConfig.then((f) => f());
      unUpdate.then((f) => f());
      unProgress.then((f) => f());
      unClose.then((f) => f());
    };
  }, [log]);

  const showUsername = config?.ui.show_username ?? true;
  const gated = !status.connected && view !== "switcher" && view !== "settings";
  const showBanner =
    update &&
    update.version !== dismissed &&
    update.version !== config?.updates.skipped_version;

  function skipUpdate(info: UpdateInfo) {
    if (!config) return;
    const next = {
      ...config,
      updates: { ...config.updates, skipped_version: info.version },
    };
    setConfigLocal(next);
    setConfig(next).catch((e) => log(`Failed to save config: ${e}`));
  }

  function install() {
    setInstalling(true);
    setProgress(null);

    installUpdate().catch((e) => {
      setInstalling(false);
      log(`Update failed: ${e}`);
    });
  }

  async function confirmClose(toTray: boolean, remember: boolean) {
    setClosePrompt(false);
    if (config && remember) {
      const next = {
        ...config,
        ui: { ...config.ui, ask_on_close: false, close_to_tray: toTray },
      };
      setConfigLocal(next);
      try {
        await setConfig(next);
      } catch (e) {
        log(`Failed to save config: ${e}`);
      }
    }
    if (toTray) getCurrentWindow().destroy().catch(() => {});
    else quit().catch(() => {});
  }

  function acceptRisk() {
    if (!config) return;
    const next = { ...config, ui: { ...config.ui, risk_accepted: true } };
    setConfigLocal(next);
    setConfig(next).catch((e) => log(`Failed to save config: ${e}`));
  }

  if (config && !config.ui.risk_accepted) {
    return (
      <div className="flex h-screen flex-col">
        <TitleBar />
        <RiskGate onAccept={acceptRisk} />
        {closePrompt && (
          <ClosePrompt
            onCancel={() => setClosePrompt(false)}
            onConfirm={confirmClose}
          />
        )}
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar active={view} onSelect={setView} />
        <div className="flex min-w-0 flex-1 flex-col">
          <StatusBar status={status} showUsername={showUsername} />
          {showBanner && (
            <UpdateBanner
              info={update}
              installing={installing}
              progress={progress}
              onInstall={install}
              onSkip={() => skipUpdate(update)}
              onDismiss={() => setDismissed(update.version)}
            />
          )}
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
      {closePrompt && (
        <ClosePrompt
          onCancel={() => setClosePrompt(false)}
          onConfirm={confirmClose}
        />
      )}
    </div>
  );
}

function ClosePrompt({
  onCancel,
  onConfirm,
}: {
  onCancel: () => void;
  onConfirm: (toTray: boolean, remember: boolean) => void;
}) {
  const [remember, setRemember] = useState(false);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/70">
      <div className="grid w-80 gap-4 rounded-lg border border-edge bg-panel p-5 shadow-lg">
        <div>
          <p className="text-sm font-semibold">Close Prowler?</p>
          <p className="mt-1 text-xs text-muted-foreground">
            Minimize to the tray to keep automations running, or quit
            completely.
          </p>
        </div>
        <div className="flex items-center justify-between gap-4">
          <span className="text-xs text-muted-foreground">
            Remember my choice
          </span>
          <Switch checked={remember} onCheckedChange={setRemember} />
        </div>
        <div className="flex justify-end gap-2">
          <Button size="sm" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => onConfirm(false, remember)}
          >
            Quit
          </Button>
          <Button size="sm" onClick={() => onConfirm(true, remember)}>
            Minimize to tray
          </Button>
        </div>
      </div>
    </div>
  );
}

function RiskGate({ onAccept }: { onAccept: () => void }) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <p className="text-lg font-semibold">Use at your own risk</p>
      <p className="max-w-md text-sm text-muted-foreground">
        Prowler is not affiliated with, endorsed, or sponsored by Riot Games.
        Using third-party tools may violate Riot's Terms of Service and could
        result in penalties, including account bans. You are solely responsible
        for any consequences of using this software.
      </p>
      <Button onClick={onAccept}>I understand and accept the risk</Button>
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
