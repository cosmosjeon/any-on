import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import NiceModal, { useModal } from '@ebay/nice-modal-react';

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { claudeAuthApi } from '@/lib/api';
import { useUserSystem } from '@/components/config-provider';
import {
  CheckCircle2,
  ExternalLink,
  Loader2,
  RefreshCw,
  XCircle,
} from 'lucide-react';

type ClaudeAuthEvent =
  | { type: 'OUTPUT'; line: string }
  | { type: 'COMPLETED'; success: boolean }
  | { type: 'ERROR'; message: string };

type ClaudeOption = {
  value: string;
  label: string;
};

type ClaudeDialogState = 'idle' | 'starting' | 'streaming' | 'success' | 'error';

const MAX_LOG_LINES = 400;
const optionRegex = /^\s*(?:\[(\d+)\]|(\d+)[\)\.])\s*(.+)$/;

const detectOption = (line: string): ClaudeOption | null => {
  const match = line.match(optionRegex);
  if (!match) return null;
  const value = match[1] ?? match[2];
  const label = match[3]?.trim();
  if (!value || !label) return null;
  return { value, label };
};

const detectLoginUrl = (line: string): string | null => {
  const matches = line.match(/https?:\/\/[^\s"'>)]+/g);
  if (!matches) return null;
  return (
    matches.find((url) => /(claude\.ai|anthropic\.com)/.test(url)) ?? null
  );
};

const statusBadge = (state: ClaudeDialogState) => {
  switch (state) {
    case 'idle':
    case 'starting':
      return { label: '준비 중', variant: 'secondary' as const, className: '' };
    case 'streaming':
      return { label: '로그인 진행 중', variant: 'default' as const, className: '' };
    case 'success':
      return {
        label: '연결 완료',
        variant: 'default' as const,
        className: 'bg-green-600 text-white hover:bg-green-600',
      };
    case 'error':
      return { label: '오류', variant: 'destructive' as const, className: '' };
    default:
      return { label: '상태 미상', variant: 'secondary' as const, className: '' };
  }
};

export const ClaudeLoginDialog = NiceModal.create(() => {
  const modal = useModal();
  const { reloadSystem } = useUserSystem();
  const [state, setState] = useState<ClaudeDialogState>('idle');
  const [logs, setLogs] = useState<string[]>([]);
  const [options, setOptions] = useState<ClaudeOption[]>([]);
  const [manualInput, setManualInput] = useState('');
  const [loginUrl, setLoginUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sendingInput, setSendingInput] = useState(false);
  const [isCancelling, setIsCancelling] = useState(false);
  const [completedMessage, setCompletedMessage] = useState<string | null>(
    null
  );

  const logContainerRef = useRef<HTMLDivElement | null>(null);
  const eventSourceRef = useRef<EventSource | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const autoOpenedUrls = useRef<Set<string>>(new Set());
  const optionCollectingRef = useRef(false);

  const cleanupStream = useCallback((shouldCancel: boolean) => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }
    const currentId = sessionIdRef.current;
    sessionIdRef.current = null;
    if (shouldCancel && currentId) {
      claudeAuthApi.cancelSession(currentId).catch(() => undefined);
    }
  }, []);

  const appendLog = useCallback((line: string) => {
    setLogs((prev) => {
      const next = [...prev, line];
      if (next.length > MAX_LOG_LINES) {
        next.splice(0, next.length - MAX_LOG_LINES);
      }
      return next;
    });
  }, []);

  useEffect(() => {
    const container = logContainerRef.current;
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  }, [logs]);

  const handleEvent = useCallback(
    (event: ClaudeAuthEvent) => {
      switch (event.type) {
        case 'OUTPUT': {
          setState((current) => (current === 'idle' ? 'streaming' : current));
          appendLog(event.line);

          const maybeOption = detectOption(event.line);
          if (maybeOption) {
            if (!optionCollectingRef.current) {
              optionCollectingRef.current = true;
              setOptions([maybeOption]);
            } else {
              setOptions((prev) => {
                const filtered = prev.filter(
                  (opt) => opt.value !== maybeOption.value
                );
                return [...filtered, maybeOption];
              });
            }
          } else if (optionCollectingRef.current && event.line.trim() !== '') {
            optionCollectingRef.current = false;
          }

          const maybeUrl = detectLoginUrl(event.line);
          if (maybeUrl) {
            setLoginUrl(maybeUrl);
          }
          break;
        }
        case 'COMPLETED': {
          cleanupStream(false);
          setState(event.success ? 'success' : 'error');
          setCompletedMessage(
            event.success
              ? 'Claude CLI 인증이 완료되었어요. 창을 닫으면 설정이 갱신됩니다.'
              : 'CLI가 비정상 종료되었습니다. 다시 시도해주세요.'
          );
          if (event.success) {
            void reloadSystem();
          }
          break;
        }
        case 'ERROR':
          cleanupStream(false);
          setState('error');
          setError(event.message || 'Claude 인증 중 오류가 발생했습니다.');
          break;
      }
    },
    [appendLog, cleanupStream, reloadSystem]
  );

  const attachStream = useCallback(
    (id: string) => {
      const source = new EventSource(`/api/auth/claude/session/${id}/stream`);
      eventSourceRef.current = source;
      source.onmessage = (evt) => {
        try {
          const payload = JSON.parse(evt.data) as ClaudeAuthEvent;
          handleEvent(payload);
        } catch (err) {
          console.warn('Failed to parse Claude auth event', err);
        }
      };
      source.onerror = () => {
        if (source.readyState === EventSource.CLOSED) return;
        setError('CLI와의 연결이 끊어졌습니다. 다시 시도해주세요.');
        setState('error');
      };
    },
    [handleEvent]
  );

  const startSession = useCallback(async () => {
    setError(null);
    setCompletedMessage(null);
    setLoginUrl(null);
    setOptions([]);
    optionCollectingRef.current = false;
    cleanupStream(true);
    setLogs([]);
    setState('starting');
    try {
      const response = await claudeAuthApi.startSession();
      sessionIdRef.current = response.session_id;
      setState('streaming');
      attachStream(response.session_id);
    } catch (err) {
      console.error('Failed to start Claude auth session', err);
      setError(err instanceof Error ? err.message : '세션 생성 실패');
      setState('error');
    }
  }, [attachStream, cleanupStream]);

  const sendInput = useCallback(
    async (value: string) => {
      if (!sessionIdRef.current) return;
      setSendingInput(true);
      try {
        await claudeAuthApi.sendInput(sessionIdRef.current, value);
        appendLog(`▶ ${value}`);
        setOptions([]);
        setManualInput('');
      } catch (err) {
        console.error('Failed to send Claude CLI input', err);
        setError(
          err instanceof Error
            ? err.message
            : 'CLI에 입력을 전달하지 못했습니다.'
        );
      } finally {
        setSendingInput(false);
      }
    },
    [appendLog]
  );

  const handleCancel = useCallback(async () => {
    if (!sessionIdRef.current) {
      modal.hide();
      modal.resolve(false);
      return;
    }
    setIsCancelling(true);
    const currentId = sessionIdRef.current;
    cleanupStream(false);
    setState('idle');
    setOptions([]);
    setLoginUrl(null);
    try {
      await claudeAuthApi.cancelSession(currentId);
    } catch (err) {
      console.warn('Failed to cancel Claude session', err);
    } finally {
      setIsCancelling(false);
    }
  }, [cleanupStream, modal]);

  const closeDialog = useCallback(() => {
    cleanupStream(true);
    modal.resolve(state === 'success');
    modal.hide();
  }, [cleanupStream, modal, state]);

  useEffect(() => {
    if (modal.visible && state === 'idle' && !sessionIdRef.current) {
      void startSession();
    }
  }, [modal.visible, startSession, state]);

  useEffect(() => () => cleanupStream(true), [cleanupStream]);

  useEffect(() => {
    if (!loginUrl || autoOpenedUrls.current.has(loginUrl)) return;
    const popup = window.open(loginUrl, '_blank', 'noopener,noreferrer');
    if (popup) {
      autoOpenedUrls.current.add(loginUrl);
    }
  }, [loginUrl]);

  const manualSendDisabled = sendingInput || !manualInput.trim();
  const badge = useMemo(() => statusBadge(state), [state]);

  const steps = useMemo(
    () => [
      {
        key: 'pick',
        label: '로그인 방법 선택',
        description:
          'Claude CLI가 제시하는 옵션 중 “브라우저로 로그인”을 고릅니다.',
        done: logs.length > 0,
      },
      {
        key: 'approve',
        label: '브라우저 승인',
        description: '새 탭에서 Claude 계정으로 로그인하고 승인합니다.',
        done: !!loginUrl || state === 'success',
      },
      {
        key: 'finish',
        label: '토큰 저장 완료',
        description:
          'CLI가 로그인을 완료하면 SecretStore에 자격 증명이 저장됩니다.',
        done: state === 'success',
      },
    ],
    [loginUrl, logs.length, state]
  );

  return (
    <Dialog open={modal.visible} onOpenChange={(open) => !open && closeDialog()}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>Claude 계정 연결</DialogTitle>
          <DialogDescription>
            터미널 없이 Claude Code CLI 로그인 절차를 웹에서 진행합니다. 아래 안내에
            따라 버튼을 누르면 Anyon이 CLI와 상호작용합니다.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="flex items-center justify-between rounded-md border px-3 py-2">
            <div className="flex items-center gap-2">
              <Badge variant={badge.variant} className={badge.className}>
                {badge.label}
              </Badge>
              {state === 'streaming' ? (
                <span className="text-sm text-muted-foreground">
                  CLI 출력과 상호작용 중입니다.
                </span>
              ) : state === 'success' ? (
                <span className="text-sm text-green-600">
                  Claude 인증이 완료되었습니다.
                </span>
              ) : state === 'error' ? (
                <span className="text-sm text-destructive">
                  오류가 발생했습니다. 다시 시도해주세요.
                </span>
              ) : (
                <span className="text-sm text-muted-foreground">
                  로그인 세션을 준비하고 있습니다.
                </span>
              )}
            </div>
            {state === 'starting' && (
              <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
            )}
          </div>

          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {completedMessage && (
            <Alert>
              <AlertDescription>{completedMessage}</AlertDescription>
            </Alert>
          )}

          <div className="grid gap-3 md:grid-cols-3">
            {steps.map((step) => (
              <div
                key={step.key}
                className={`rounded-md border px-3 py-2 ${
                  step.done
                    ? 'border-green-600 bg-green-50 dark:bg-green-950/30'
                    : ''
                }`}
              >
                <div className="flex items-center gap-2 text-sm font-medium">
                  {step.done ? (
                    <CheckCircle2 className="h-4 w-4 text-green-600" />
                  ) : (
                    <Loader2 className="h-4 w-4 text-muted-foreground" />
                  )}
                  {step.label}
                </div>
                <p className="mt-1 text-xs text-muted-foreground">
                  {step.description}
                </p>
              </div>
            ))}
          </div>

          {loginUrl && (
            <div className="rounded-md border bg-muted/50 px-3 py-2">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div>
                  <p className="text-sm font-medium">브라우저 승인이 필요합니다</p>
                  <p className="text-xs text-muted-foreground break-all">
                    {loginUrl}
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    window.open(loginUrl, '_blank', 'noopener,noreferrer')
                  }
                  className="flex items-center gap-1"
                >
                  <ExternalLink className="h-3.5 w-3.5" /> 새 탭에서 열기
                </Button>
              </div>
            </div>
          )}

          <div>
            <div className="flex items-center justify-between">
              <p className="text-sm font-medium">CLI 출력</p>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  if (!logs.length) return;
                  navigator.clipboard?.writeText(logs.join('\n')).catch(() => undefined);
                }}
              >
                로그 복사
              </Button>
            </div>
            <div
              ref={logContainerRef}
              className="mt-2 h-56 overflow-y-auto rounded-md border bg-muted/40 px-3 py-2 font-mono text-xs"
            >
              {logs.length === 0 ? (
                <p className="text-muted-foreground">CLI 응답을 기다리는 중…</p>
              ) : (
                logs.map((line, idx) => <p key={`${line}-${idx}`}>{line}</p>)
              )}
            </div>
          </div>

          {options.length > 0 && (
            <div>
              <p className="mb-2 text-sm font-medium">CLI가 제시한 선택지</p>
              <div className="flex flex-wrap gap-2">
                {options.map((option) => (
                  <Button
                    key={option.value}
                    variant="outline"
                    disabled={sendingInput}
                    onClick={() => void sendInput(option.value)}
                  >
                    {option.value}. {option.label}
                  </Button>
                ))}
              </div>
            </div>
          )}

          <div className="flex items-center gap-2">
            <Input
              placeholder="직접 값을 입력해 전송 (숫자 또는 답변)"
              value={manualInput}
              onChange={(e) => setManualInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !manualSendDisabled) {
                  e.preventDefault();
                  void sendInput(manualInput.trim());
                }
              }}
            />
            <Button
              disabled={manualSendDisabled}
              onClick={() => void sendInput(manualInput.trim())}
            >
              전송
            </Button>
          </div>
        </div>

        <DialogFooter className="flex flex-col gap-2 sm:flex-row sm:justify-between">
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={() => void startSession()}>
              <RefreshCw className="mr-1 h-4 w-4" /> 다시 시도
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void handleCancel()}
              disabled={isCancelling}
            >
              <XCircle className="mr-1 h-4 w-4" /> 세션 중단
            </Button>
          </div>
          <Button onClick={closeDialog} disabled={state !== 'success'}>
            완료
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
});

export default ClaudeLoginDialog;
