import { useEffect, useState, type ReactNode } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";

const appWindow = getCurrentWindow();

export function TitleBar() {
  const [maximized, setMaximized] = useState(false);
  const [version, setVersion] = useState("");

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const sync = () => appWindow.isMaximized().then(setMaximized).catch(() => {});
    sync();
    appWindow.onResized(sync).then((fn) => (unlisten = fn));
    getVersion().then(setVersion).catch(() => {});
    return () => unlisten?.();
  }, []);

  return (
    <header
      data-tauri-drag-region
      className="flex h-9 shrink-0 select-none items-center justify-between border-b border-edge bg-panel"
    >
      <div className="pointer-events-none flex items-center gap-2 pl-3">
        <img src="/logo.png" alt="Prowler" className="h-5 w-5 rounded" />
        <span className="text-xs font-semibold tracking-tight text-foreground">
          Prowler
          <span className="ml-1 text-muted-foreground">{version && `v${version}`}</span>
        </span>
      </div>
      <div className="flex h-full">
        <TitleBarButton label="Minimize" onClick={() => appWindow.minimize()}>
          <Minus className="h-3.5 w-3.5" />
        </TitleBarButton>
        <TitleBarButton
          label={maximized ? "Restore" : "Maximize"}
          onClick={() => appWindow.toggleMaximize()}
        >
          {maximized ? <Copy className="h-3 w-3 -scale-x-100" /> : <Square className="h-3 w-3" />}
        </TitleBarButton>
        <TitleBarButton close label="Close" onClick={() => appWindow.close()}>
          <X className="h-4 w-4" />
        </TitleBarButton>
      </div>
    </header>
  );
}

function TitleBarButton({
  close,
  label,
  onClick,
  children,
}: {
  close?: boolean;
  label: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      tabIndex={-1}
      aria-label={label}
      onClick={onClick}
      className={`inline-flex h-full w-12 items-center justify-center text-muted-foreground transition-colors ${
        close
          ? "hover:bg-[#e81123] hover:text-white"
          : "hover:bg-panel2 hover:text-foreground"
      }`}
    >
      {children}
    </button>
  );
}
