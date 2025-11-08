import { useEffect, useMemo, useRef, useState } from 'react';
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
import { Input } from '@/components/ui/input';
import { Loader } from '@/components/ui/loader';
import { claudeAuthApi } from '@/lib/api';
import { useUserSystem } from '@/components/config-provider';
import { useTranslation } from 'react-i18next';

interface ClaudeAuthEventPayload {
  type: 'OUTPUT' | 'COMPLETED' | 'ERROR';
  line?: string;
  message?: string;
  success?: boolean;
}

interface LogEntry {
  id: number;
  text: string;
  url?: string;
}

const urlRegex = /(https?:\/\/\S+)/i;

export const ClaudeLoginDialog = NiceModal.create(() => {
  const modal = useModal();
  const { reloadSystem } = useUserSystem();
  const { t } = useTranslation(['settings']);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [input, setInput] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<'idle' | 'running' | 'success' | 'error'>('idle');
  const eventSourceRef = useRef<EventSource | null>(null);
  const logIdRef = useRef(0);

  useEffect(() => {
    if (!modal.visible) return;
    const start = async () => {
      try {
        setStatus('running');
        const response = await claudeAuthApi.startSession();
        setSessionId(response.session_id);
        setLogs([]);
        setError(null);
      } catch (err: any) {
        setError(err?.message || 'Failed to start Claude login.');
        setStatus('error');
      }
    };
    start();
    return () => {
      if (sessionId) {
        claudeAuthApi.cancelSession(sessionId).catch(() => {
          /* ignore */
        });
      }
      eventSourceRef.current?.close();
    };
  }, [modal.visible]);

  useEffect(() => {
    if (!sessionId) return;
    const source = new EventSource(`/api/auth/claude/session/${sessionId}/stream`);
    eventSourceRef.current = source;
    source.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data) as ClaudeAuthEventPayload;
        if (payload.type === 'OUTPUT' && payload.line) {
          const match = payload.line.match(urlRegex);
          const entry: LogEntry = {
            id: logIdRef.current++,
            text: payload.line,
            url: match ? match[0] : undefined,
          };
          setLogs((prev) => [...prev, entry]);
        } else if (payload.type === 'COMPLETED') {
          setStatus('success');
          reloadSystem();
          setTimeout(() => {
            modal.resolve(true);
            modal.hide();
          }, 1200);
        } else if (payload.type === 'ERROR' && payload.message) {
          setError(payload.message);
          setStatus('error');
        }
      } catch (err) {
        console.error('Failed to parse Claude event', err);
      }
    };
    source.onerror = () => {
      setError('Connection lost. Please try again.');
      setStatus('error');
      source.close();
    };
    return () => source.close();
  }, [sessionId, modal, reloadSystem]);

  const latestUrl = useMemo(() => {
    for (let i = logs.length - 1; i >= 0; i -= 1) {
      if (logs[i].url) {
        return logs[i].url;
      }
    }
    return null;
  }, [logs]);

  const handleSend = async () => {
    if (!sessionId || !input.trim()) return;
    try {
      await claudeAuthApi.sendInput(sessionId, input.trim());
      setInput('');
    } catch (err: any) {
      setError(err?.message || 'Failed to send input.');
    }
  };

  const handleClose = () => {
    modal.resolve(false);
    modal.hide();
  };

  return (
    <Dialog open={modal.visible} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>{t('settings.general.claude.dialog.title')}</DialogTitle>
          <DialogDescription>
            {t('settings.general.claude.dialog.description')}
          </DialogDescription>
        </DialogHeader>

        {status === 'idle' || status === 'running' ? (
          <div className="space-y-3">
            <div className="border rounded-md bg-muted/30 p-3 h-56 overflow-auto text-sm font-mono">
              {logs.length === 0 ? (
                <Loader message={t('settings.general.claude.dialog.waiting')} />
              ) : (
                logs.map((log) => (
                  <div key={log.id} className="whitespace-pre-wrap">
                    {log.text}
                  </div>
                ))
              )}
            </div>
            {latestUrl && (
              <Button
                variant="outline"
                onClick={() => window.open(latestUrl, '_blank')}
                className="w-full"
              >
                {t('settings.general.claude.dialog.openLink')}
              </Button>
            )}
            <div className="flex gap-2">
              <Input
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder={t('settings.general.claude.dialog.inputPlaceholder')}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault();
                    handleSend();
                  }
                }}
              />
              <Button onClick={handleSend} disabled={!sessionId || !input.trim()}>
                {t('settings.general.claude.dialog.send')}
              </Button>
            </div>
          </div>
        ) : null}

        {status === 'success' && (
          <Alert>
            <AlertDescription>
              {t('settings.general.claude.dialog.success')}
            </AlertDescription>
          </Alert>
        )}

        {status === 'error' && error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            {t('settings.general.claude.dialog.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
});

export default ClaudeLoginDialog;
