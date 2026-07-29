import { useEffect, useRef } from "react";
import { useLog, type LogLevel } from "../lib/log";
import { Button } from "@/components/ui/button";

const DOT_COLOR: Record<LogLevel, string> = {
  info: "bg-info",
  success: "bg-good",
  warn: "bg-warn",
  error: "bg-bad",
};

export function LogFeed() {
  const { logs, clear } = useLog();
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <aside className="flex h-48 shrink-0 flex-col border-t border-edge bg-panel">
      <div className="flex items-center justify-between border-b border-edge px-4 py-2">
        <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Activity log
        </span>
        <Button
          variant="ghost"
          size="xs"
          onClick={clear}
          className="text-muted-foreground hover:text-foreground"
        >
          Clear
        </Button>
      </div>
      <div className="flex-1 overflow-y-auto px-4 py-2 text-xs leading-relaxed">
        {logs.length === 0 ? (
          <p className="text-muted-foreground">No activity yet.</p>
        ) : (
          logs.map((l) => (
            <div key={l.id} className="mb-1 flex items-center gap-2">
              <span className="shrink-0 tabular-nums text-muted-foreground">
                {l.time}
              </span>
              <span
                className={`size-1.5 shrink-0 rounded-full ${DOT_COLOR[l.level]}`}
              />
              <span className="min-w-0 truncate">{l.text}</span>
            </div>
          ))
        )}
        <div ref={endRef} />
      </div>
    </aside>
  );
}
