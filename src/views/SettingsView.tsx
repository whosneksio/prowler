import { useEffect, useState } from "react";
import { Card, ViewHeader } from "../components/common";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { getConfig, setConfig } from "../lib/ipc";
import { useLog } from "../lib/log";
import type { Config } from "../lib/types";

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
            <span className="ml-1 text-xs text-muted-foreground">
              Temporarily disabled.
            </span>
          </div>
        </Card>

        <Card title="Action delays" desc="Seconds to wait before each automated action (0-2s).">
          <div className="grid gap-3">
            <DelayRow
              label="Auto Accept"
              value={config.auto_accept.delay_seconds}
              onChange={(v) =>
                save({
                  ...config,
                  auto_accept: { ...config.auto_accept, delay_seconds: v },
                })
              }
            />
            <DelayRow
              label="Instalock"
              value={config.instalock.delay_seconds}
              onChange={(v) =>
                save({
                  ...config,
                  instalock: { ...config.instalock, delay_seconds: v },
                })
              }
            />
            <DelayRow
              label="Autoban"
              value={config.autoban.delay_seconds}
              onChange={(v) =>
                save({
                  ...config,
                  autoban: { ...config.autoban, delay_seconds: v },
                })
              }
            />
          </div>
        </Card>
      </div>
    </div>
  );
}

function DelayRow({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
}) {
  const [drag, setDrag] = useState<number | null>(null);
  const shown = drag ?? value;

  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-sm">{label}</span>
      <div className="flex items-center gap-3">
        <Slider
          min={0}
          max={2}
          step={0.1}
          value={[shown]}
          onValueChange={([v]) => setDrag(v)}
          onValueCommit={([v]) => {
            setDrag(null);
            onChange(v);
          }}
          className="w-40"
        />
        <span className="w-10 text-right text-sm text-muted-foreground">
          {shown.toFixed(1)}s
        </span>
      </div>
    </div>
  );
}
