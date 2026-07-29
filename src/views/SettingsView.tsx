import { useEffect, useState } from "react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { getConfig, setConfig } from "../lib/ipc";
import { useLog } from "../lib/log";
import type { Config, DelayRange } from "../lib/types";

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
