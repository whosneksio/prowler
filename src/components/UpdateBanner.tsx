import { X } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import type { UpdateInfo, UpdateProgress } from "@/lib/types";

export function UpdateBanner({
  info,
  installing,
  progress,
  onInstall,
  onSkip,
  onDismiss,
}: {
  info: UpdateInfo;
  installing: boolean;
  progress: UpdateProgress | null;
  onInstall: () => void;
  onSkip: () => void;
  onDismiss: () => void;
}) {
  const pct =
    progress && progress.total
      ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
      : null;

  return (
    <Alert className="flex items-center gap-3 rounded-none border-x-0 border-t-0 border-edge bg-panel py-2">
      <AlertDescription className="flex min-w-0 flex-1 flex-row items-center gap-3 text-xs">
        {installing ? (
          <>
            <span className="shrink-0">
              Installing update {info.version}…
            </span>
            <div className="h-1.5 min-w-0 flex-1 overflow-hidden rounded bg-panel2">
              <div
                className="h-full bg-primary transition-[width]"
                style={{ width: `${pct ?? 0}%` }}
              />
            </div>
            <span className="shrink-0 tabular-nums text-muted-foreground">
              {pct !== null ? `${pct}%` : "…"}
            </span>
          </>
        ) : (
          <>
            <span className="min-w-0 flex-1 truncate">
              Update {info.version} is available (you have{" "}
              {info.current_version}).
            </span>
            <Button size="sm" className="h-6 px-2 text-xs" onClick={onInstall}>
              Update now
            </Button>
            <Button
              size="sm"
              variant="ghost"
              className="h-6 px-2 text-xs text-muted-foreground"
              onClick={onSkip}
            >
              Skip {info.version}
            </Button>
            <button
              aria-label="Dismiss"
              onClick={onDismiss}
              className="text-muted-foreground transition-colors hover:text-foreground"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          </>
        )}
      </AlertDescription>
    </Alert>
  );
}
