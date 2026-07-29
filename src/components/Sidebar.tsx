import {
  ArrowLeftRight,
  Settings,
  Sparkles,
  Users,
  Wrench,
  Zap,
  type LucideIcon,
} from "lucide-react";
import type { ViewId } from "../lib/types";

const NAV: { id: ViewId; label: string; icon: LucideIcon }[] = [
  { id: "switcher", label: "Accounts", icon: ArrowLeftRight },
  { id: "automation", label: "Automation", icon: Zap },
  { id: "customization", label: "Profile", icon: Sparkles },
  { id: "runes", label: "Runes", icon: Zap },
  { id: "tools", label: "Game Tools", icon: Wrench },
  { id: "social", label: "Social", icon: Users },
  { id: "settings", label: "Settings", icon: Settings },
];

export function Sidebar({
  active,
  onSelect,
}: {
  active: ViewId;
  onSelect: (v: ViewId) => void;
}) {
  return (
    <nav className="flex w-52 shrink-0 flex-col border-r border-edge bg-panel">
      <div className="flex flex-col gap-1 px-3 pt-4">
        {NAV.map((item) => {
          const isActive = item.id === active;
          return (
            <button
              key={item.id}
              onClick={() => onSelect(item.id)}
              className={`flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors ${
                isActive
                  ? "bg-primary/15 text-text"
                  : "text-muted-foreground hover:bg-panel2 hover:text-text"
              }`}
            >
              <item.icon className="h-4 w-4 shrink-0" />
              {item.label}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
