import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { onLog } from "./ipc";

export type LogLevel = "info" | "success" | "warn" | "error";

export interface LogEntry {
  id: number;
  time: string;
  level: LogLevel;
  text: string;
}

interface LogCtx {
  logs: LogEntry[];
  log: (text: string, level?: LogLevel) => void;
  clear: () => void;
}

const Ctx = createContext<LogCtx | null>(null);

function inferLevel(text: string): LogLevel {
  if (/fail|error|unable|denied|invalid/i.test(text)) return "error";
  if (/disconnect|warn|retry|timeout/i.test(text)) return "warn";
  if (/connected|saved|success|done|complete|switched|deleted|renamed/i.test(text))
    return "success";
  return "info";
}

export function LogProvider({ children }: { children: ReactNode }) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const idRef = useRef(0);

  const log = useCallback((text: string, level?: LogLevel) => {
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => {
      const next = [
        ...prev,
        { id: idRef.current++, time, level: level ?? inferLevel(text), text },
      ];
      return next.slice(-200);
    });
  }, []);

  useEffect(() => {
    const un = onLog((line) => log(line));
    return () => {
      un.then((f) => f());
    };
  }, [log]);

  const clear = useCallback(() => setLogs([]), []);

  const value = useMemo(() => ({ logs, log, clear }), [logs, log, clear]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useLog(): LogCtx {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useLog must be used within LogProvider");
  return ctx;
}
