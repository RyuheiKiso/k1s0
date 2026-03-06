import { useEffect, useRef } from "react";

interface LogEntry {
  timestamp: string;
  message: string;
}

interface LiveLogProps {
  logs: LogEntry[];
  connected: boolean;
  onClear: () => void;
}

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleTimeString("ja-JP", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return "";
  }
}

function colorize(message: string): string {
  if (message.includes("Error") || message.includes("503") || message.includes("error"))
    return "text-red-400";
  if (message.includes("200") || message.includes("applied") || message.includes("started"))
    return "text-green-400";
  if (message.includes("Resetting") || message.includes("removed") || message.includes("stopped"))
    return "text-yellow-400";
  return "text-slate-300";
}

export default function LiveLog({ logs, connected, onClear }: LiveLogProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-slate-700">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
            Live Log
          </span>
          <span
            className={`inline-block w-2 h-2 rounded-full ${
              connected ? "bg-green-400" : "bg-red-400"
            }`}
          />
        </div>
        <button
          onClick={onClear}
          className="text-xs text-slate-500 hover:text-slate-300 transition-colors"
        >
          Clear
        </button>
      </div>
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-2 font-mono text-xs leading-relaxed log-panel"
      >
        {logs.length === 0 ? (
          <div className="text-slate-600 italic">
            Waiting for log stream...
          </div>
        ) : (
          logs.map((entry, i) => (
            <div key={i} className="flex gap-2 hover:bg-slate-800/50">
              <span className="text-slate-600 shrink-0">
                {formatTime(entry.timestamp)}
              </span>
              <span className={colorize(entry.message)}>
                {entry.message}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
