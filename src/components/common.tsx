import type { ReactNode } from "react";
import { TriangleAlert } from "lucide-react";
import {
  Card as UICard,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { cn } from "@/lib/utils";

export function ViewHeader({
  title,
  subtitle,
  action,
}: {
  title: ReactNode;
  subtitle?: string;
  action?: ReactNode;
}) {
  return (
    <div className="mb-6 flex items-start justify-between">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
        {subtitle && <p className="mt-1 text-sm text-muted-foreground">{subtitle}</p>}
      </div>
      {action}
    </div>
  );
}

export function Card({
  title,
  desc,
  children,
  className,
  contentClassName,
  action,
}: {
  title?: string;
  desc?: string;
  children?: ReactNode;
  className?: string;
  contentClassName?: string;
  action?: ReactNode;
}) {
  const hasHeader = title || desc || action;
  return (
    <UICard
      className={cn("gap-0 border-edge bg-panel p-5 shadow-none", className)}
    >
      {hasHeader && (
        <CardHeader className="gap-0 p-0">
          <div className="flex items-start justify-between gap-4">
            <div className="grid gap-1">
              {title && <CardTitle className="text-sm">{title}</CardTitle>}
              {desc && (
                <CardDescription className="text-xs">{desc}</CardDescription>
              )}
            </div>
            {action && <div className="shrink-0">{action}</div>}
          </div>
        </CardHeader>
      )}
      {children && (
        <CardContent
          className={cn("p-0", hasHeader && "mt-4", contentClassName)}
        >
          {children}
        </CardContent>
      )}
    </UICard>
  );
}

export function GuardBanner({ children }: { children: ReactNode }) {
  return (
    <Alert className="mb-5 border-warn/30 bg-warn/10 py-2.5 text-warn">
      <TriangleAlert />
      <AlertDescription className="text-xs text-warn">
        {children}
      </AlertDescription>
    </Alert>
  );
}
