import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { checkUpdate, getConfig, installUpdate, setConfig } from "../lib/ipc";
import { useLog } from "../lib/log";
import type { Config, DelayRange, UpdateInfo } from "../lib/types";

const PROVIDERS = [
  { id: "opgg", label: "OP.GG" },
  { id: "porofessor", label: "Porofessor" },
  { id: "ugg", label: "U.GG" },
];

export function SettingsView() {
  const { log } = useLog();
  const [config, setLocal] = useState<Config | null>(null);

  useEffect(() => {
    getConfig().then(setLocal).catch((e) => log(`Failed to load config: ${e}`));
  }, [log]);

  async function save(next: Config) {
    setLocal(next);
    try {
      await setConfig(next);
    } catch (e) {
      log(`Failed to save config: ${e}`);
    }
  }

  if (!config) {
    return (
      <div className="flex h-full flex-col">
        <ViewHeader title="Settings" />
        <p className="text-sm text-muted-foreground">Loading…</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader title="Settings" subtitle="Preferences are saved automatically." />
      <div className="grid max-w-xl gap-4">
        <UpdatesCard config={config} save={save} />

        <Card
          title="Privacy"
          desc="Adjust whether your username is shown."
        >
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm text-muted-foreground">
              Show username
            </span>
            <Switch
              checked={config.ui.show_username}
              onCheckedChange={(on) =>
                save({ ...config, ui: { ...config.ui, show_username: on } })
              }
            />
          </div>
        </Card>

        <Card
          title="Window"
          desc="What happens when you close the window."
        >
          <div className="grid gap-3">
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm text-muted-foreground">
                Ask before closing
              </span>
              <Switch
                checked={config.ui.ask_on_close}
                onCheckedChange={(on) =>
                  save({ ...config, ui: { ...config.ui, ask_on_close: on } })
                }
              />
            </div>
            <div
              className={`flex items-center justify-between gap-4 ${
                config.ui.ask_on_close ? "opacity-50" : ""
              }`}
            >
              <span className="text-sm text-muted-foreground">
                Close to tray
              </span>
              <Switch
                disabled={config.ui.ask_on_close}
                checked={config.ui.close_to_tray}
                onCheckedChange={(on) =>
                  save({ ...config, ui: { ...config.ui, close_to_tray: on } })
                }
              />
            </div>
          </div>
        </Card>

        <Card
          title="Lobby Reveal"
          desc="Which site opens when you reveal a lobby."
          className="opacity-50"
        >
          <div className="flex items-center gap-2">
            {PROVIDERS.map((p) => (
              <Button
                key={p.id}
                size="sm"
                disabled
                variant={
                  config.lobby_reveal.provider === p.id ? "default" : "secondary"
                }
              >
                {p.label}
              </Button>
            ))}
          </div>
        </Card>

        <Card
          title="Action delays"
          desc="Each action waits a random time within its window before firing."
        >
          <div className="grid gap-3">
            <DelayRow
              label="Auto Accept"
              value={config.auto_accept.delay}
              onChange={(v) =>
                save({
                  ...config,
                  auto_accept: { ...config.auto_accept, delay: v },
                })
              }
            />
            <DelayRow
              label="Instalock"
              value={config.instalock.delay}
              onChange={(v) =>
                save({
                  ...config,
                  instalock: { ...config.instalock, delay: v },
                })
              }
            />
            <DelayRow
              label="Autoban"
              value={config.autoban.delay}
              onChange={(v) =>
                save({
                  ...config,
                  autoban: { ...config.autoban, delay: v },
                })
              }
            />
          </div>
        </Card>
      </div>
    </div>
  );
}

function UpdatesCard({
  config,
  save,
}: {
  config: Config;
  save: (next: Config) => void;
}) {
  const { log } = useLog();
  const [version, setVersion] = useState("");
  const [checking, setChecking] = useState(false);

  const [result, setResult] = useState<UpdateInfo | null | undefined>(undefined);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
  }, []);

  async function check() {
    setChecking(true);
    try {
      setResult(await checkUpdate());
    } catch (e) {
      log(`Update check failed: ${e}`);
    } finally {
      setChecking(false);
    }
  }

  function install() {
    setInstalling(true);
    installUpdate().catch((e) => {
      setInstalling(false);
      log(`Update failed: ${e}`);
    });
  }

  return (
    <Card title="Updates" desc={version ? `You are on version ${version}.` : undefined}>
      <div className="grid gap-3">
        <div className="flex items-center justify-between gap-4">
          <span className="text-sm text-muted-foreground">
            Check for updates automatically
          </span>
          <Switch
            checked={config.updates.auto_check}
            onCheckedChange={(on) =>
              save({
                ...config,
                updates: { ...config.updates, auto_check: on },
              })
            }
          />
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant="secondary" disabled={checking} onClick={check}>
            {checking ? "Checking…" : "Check for updates"}
          </Button>
          {config.updates.skipped_version && (
            <Button
              size="sm"
              variant="ghost"
              className="text-muted-foreground"
              onClick={() =>
                save({
                  ...config,
                  updates: { ...config.updates, skipped_version: null },
                })
              }
            >
              Stop skipping v{config.updates.skipped_version}
            </Button>
          )}
        </div>
        {result === null && (
          <p className="text-xs text-muted-foreground">Prowler is up to date.</p>
        )}
        {result && (
          <div className="grid gap-2 rounded-md border border-edge bg-panel2 p-3">
            <p className="text-sm">
              Version {result.version} is available
              {result.date ? ` (${result.date.slice(0, 10)})` : ""}.
            </p>
            {result.notes && (
              <p className="whitespace-pre-wrap text-xs text-muted-foreground">
                {result.notes}
              </p>
            )}
            <div>
              <Button size="sm" disabled={installing} onClick={install}>
                {installing ? "Installing…" : "Download and install"}
              </Button>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}

const DELAY_MAX = 20;

function DelayRow({
  label,
  value,
  onChange,
}: {
  label: string;
  value: DelayRange;
  onChange: (v: DelayRange) => void;
}) {
  const [drag, setDrag] = useState<[number, number] | null>(null);
  const [min, max] = drag ?? [value.min, value.max];

  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-sm">{label}</span>
      <div className="flex items-center gap-3">
        <Slider
          min={0}
          max={DELAY_MAX}
          step={0.5}
          minStepsBetweenThumbs={0}
          value={[min, max]}
          onValueChange={([lo, hi]) => setDrag([lo, hi])}
          onValueCommit={([lo, hi]) => {
            setDrag(null);
            onChange({ min: lo, max: hi });
          }}
          className="w-40"
        />
        <span className="w-20 text-right text-sm tabular-nums text-muted-foreground">
          {min.toFixed(1)}-{max.toFixed(1)}s
        </span>
      </div>
    </div>
  );
}
