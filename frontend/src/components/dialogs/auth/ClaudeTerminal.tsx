import { useCallback, useEffect, useRef, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import type { IDisposable } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { claudeAuthApi, ClaudePtyLogEntry } from '@/lib/api';

interface ClaudeTerminalProps {
  onClose?: () => void;
  onSuccess?: () => void;
}

const SUCCESS_MARKERS = ['로그인 성공', 'Credential이 저장'];
const MAX_LIVE_LOG_LENGTH = 8000;

export function ClaudeTerminal({ onClose, onSuccess }: ClaudeTerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const successHandledRef = useRef(false);
  const decoderRef = useRef<TextDecoder | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const hasFetchedLogsRef = useRef(false);
  const reconnectTimerRef = useRef<number | null>(null);
  const unmountedRef = useRef(false);
  const terminalDataDisposableRef = useRef<IDisposable | null>(null);

  const [sessionId, setSessionId] = useState<string | null>(null);
  const [liveLog, setLiveLog] = useState('');
  const [showLogs, setShowLogs] = useState(false);
  const [serverLogs, setServerLogs] = useState<ClaudePtyLogEntry[]>([]);
  const [logLoading, setLogLoading] = useState(false);
  const [logError, setLogError] = useState<string | null>(null);

  const appendLiveLog = useCallback((chunk: string) => {
    if (!chunk) return;
    setLiveLog((prev) => {
      let next = prev + chunk;
      if (next.length > MAX_LIVE_LOG_LENGTH) {
        next = next.slice(next.length - MAX_LIVE_LOG_LENGTH);
      }
      return next;
    });
  }, []);

  const fetchLogs = useCallback(async () => {
    const id = sessionIdRef.current;
    if (!id) return;
    setLogLoading(true);
    setLogError(null);
    try {
      const entries = await claudeAuthApi.getPtySessionLog(id);
      setServerLogs(entries);
      hasFetchedLogsRef.current = true;
    } catch (error) {
      setLogError(
        error instanceof Error
          ? error.message
          : '로그를 불러오지 못했습니다.'
      );
    } finally {
      setLogLoading(false);
    }
  }, []);

  const connectWebSocket = useCallback(() => {
    if (unmountedRef.current) return;

    if (wsRef.current) {
      const state = wsRef.current.readyState;
      if (state === WebSocket.OPEN || state === WebSocket.CONNECTING) {
        return;
      }
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/auth/claude/pty`;
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;
    successHandledRef.current = false;
    decoderRef.current = null;

    ws.binaryType = 'arraybuffer';

    ws.onopen = () => {
      termRef.current?.writeln('Connected to Claude Code login...\r\n');
    };

    ws.onmessage = (event) => {
      if (!termRef.current) return;
      if (event.data instanceof ArrayBuffer) {
        const data = new Uint8Array(event.data);
        termRef.current.write(data);
        const decoder = decoderRef.current ?? new TextDecoder();
        decoderRef.current = decoder;
        const decoded = decoder.decode(data, { stream: true });
        appendLiveLog(decoded);
      } else if (typeof event.data === 'string') {
        if (event.data.startsWith('__CLAUDE_META__:')) {
          const metaPayload = event.data.replace('__CLAUDE_META__:', '');
          try {
            const parsed = JSON.parse(metaPayload) as { sessionId?: string };
            if (parsed.sessionId) {
              setSessionId(parsed.sessionId);
              sessionIdRef.current = parsed.sessionId;
              hasFetchedLogsRef.current = false;
              setServerLogs([]);
            }
          } catch (err) {
            console.warn('Failed to parse Claude PTY metadata', err);
          }
          return;
        }

        termRef.current.write(event.data);
        appendLiveLog(event.data);

        if (
          !successHandledRef.current &&
          SUCCESS_MARKERS.some((marker) => event.data.includes(marker))
        ) {
          successHandledRef.current = true;
          setTimeout(() => {
            if (
              ws.readyState === WebSocket.OPEN ||
              ws.readyState === WebSocket.CONNECTING
            ) {
              ws.close();
            }
            onSuccess?.();
          }, 500);
        }
      }
    };

    ws.onerror = (error) => {
      termRef.current?.writeln(`\r\nWebSocket error: ${error}\r\n`);
    };

    ws.onclose = () => {
      const decoder = decoderRef.current;
      if (decoder) {
        const remaining = decoder.decode();
        appendLiveLog(remaining);
      }
      termRef.current?.writeln('\r\nConnection closed.\r\n');

      if (!hasFetchedLogsRef.current && sessionIdRef.current) {
        void fetchLogs();
      }

      if (!unmountedRef.current && !successHandledRef.current) {
        if (reconnectTimerRef.current) {
          window.clearTimeout(reconnectTimerRef.current);
        }
        reconnectTimerRef.current = window.setTimeout(() => {
          connectWebSocket();
        }, 1000);
      }
    };

    if (termRef.current) {
      terminalDataDisposableRef.current?.dispose();
      terminalDataDisposableRef.current = termRef.current.onData((data) => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(data);
        }
      });
    }
  }, [appendLiveLog, fetchLogs, onSuccess]);

  useEffect(() => {
    setLiveLog('');
    setServerLogs([]);
    setShowLogs(false);
    setLogError(null);
    setSessionId(null);
    sessionIdRef.current = null;
    hasFetchedLogsRef.current = false;
    decoderRef.current = null;
    unmountedRef.current = false;

    if (!terminalRef.current) return;
    if (termRef.current) {
      connectWebSocket();
      return;
    }

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
      },
      rows: 24,
      cols: 80,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalRef.current);
    requestAnimationFrame(() => {
      try {
        fitAddon.fit();
      } catch (err) {
        console.warn('Failed to fit terminal', err);
      }
    });

    termRef.current = term;

    const handleResize = () => {
      try {
        fitAddon.fit();
      } catch {
        // ignore fit errors during resize
      }
    };
    window.addEventListener('resize', handleResize);

    connectWebSocket();

    return () => {
      unmountedRef.current = true;
      window.removeEventListener('resize', handleResize);
      if (reconnectTimerRef.current) {
        window.clearTimeout(reconnectTimerRef.current);
      }
      terminalDataDisposableRef.current?.dispose();
      terminalDataDisposableRef.current = null;
      const ws = wsRef.current;
      if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
        ws.close();
      }
      termRef.current?.dispose();
      termRef.current = null;
    };
  }, [connectWebSocket]);

  useEffect(() => {
    if (showLogs && sessionIdRef.current && !hasFetchedLogsRef.current) {
      void fetchLogs();
    }
  }, [fetchLogs, showLogs]);

  return (
    <div className="flex flex-col h-full">
      <div ref={terminalRef} className="flex-1" />
      {showLogs && (
        <div className="border-t border-border bg-muted/10 p-4 text-xs text-foreground space-y-4 max-h-64 overflow-y-auto">
          <div>
            <div className="flex items-center justify-between">
              <span className="font-medium text-sm text-foreground">
                실시간 출력
              </span>
              <button
                type="button"
                onClick={() => {
                  setLiveLog('');
                }}
                className="text-[11px] text-muted-foreground hover:text-foreground"
              >
                로그 지우기
              </button>
            </div>
            <pre className="mt-2 whitespace-pre-wrap break-words rounded bg-background/70 p-2 text-foreground/90">
              {liveLog.trim().length ? liveLog : '출력이 없습니다.'}
            </pre>
          </div>
          <div>
            <div className="flex items-center justify-between gap-2">
              <span className="font-medium text-sm text-foreground">
                세션 로그 기록
              </span>
              <div className="flex items-center gap-2">
                {logError && (
                  <span className="text-[11px] text-destructive">{logError}</span>
                )}
                <button
                  type="button"
                  onClick={() => void fetchLogs()}
                  className="rounded border border-border px-2 py-1 text-[11px] hover:bg-background"
                  disabled={logLoading || !sessionId}
                >
                  {logLoading ? '불러오는 중…' : '새로고침'}
                </button>
              </div>
            </div>
            <div className="mt-2 space-y-1">
              {serverLogs.length === 0 && !logLoading ? (
                <p className="text-muted-foreground">
                  아직 저장된 로그가 없습니다.
                </p>
              ) : (
                serverLogs.map((entry, index) => {
                  const time = new Date(entry.timestamp * 1000).toLocaleTimeString();
                  const label =
                    entry.direction === 'input' ? '입력' : '출력';
                  const colorClass =
                    entry.direction === 'input'
                      ? 'text-emerald-300'
                      : 'text-sky-300';
                  return (
                    <div key={`${entry.timestamp}-${index}`} className="space-x-2">
                      <span className="text-muted-foreground">{time}</span>
                      <span className={colorClass}>{label}</span>
                      <span className="whitespace-pre-wrap break-words text-foreground">
                        {entry.data || '(빈 입력)'}
                      </span>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        </div>
      )}
      <div className="flex flex-col gap-3 border-t border-border bg-background/60 p-4">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setShowLogs((prev) => !prev)}
              className="rounded border border-border px-3 py-1.5 text-sm text-foreground hover:bg-background"
            >
              {showLogs ? '로그 숨기기' : '로그 보기'}
            </button>
            {sessionId && (
              <span className="text-[11px] text-muted-foreground">
                세션 ID: {sessionId}
              </span>
            )}
            {logLoading && (
              <span className="text-[11px] text-muted-foreground">
                로그 불러오는 중…
              </span>
            )}
          </div>
          {onClose && (
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-600 hover:bg-gray-700 rounded text-white"
            >
              Close
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
