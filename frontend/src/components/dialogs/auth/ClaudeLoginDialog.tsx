import NiceModal, { useModal } from '@ebay/nice-modal-react';

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ClaudeTerminal } from './ClaudeTerminal';

export const ClaudeLoginDialog = NiceModal.create(() => {
  const modal = useModal();

  return (
    <Dialog open={modal.visible} onOpenChange={(open) => !open && modal.remove()}>
      <DialogContent className="max-w-6xl h-[85vh] flex flex-col">
        <DialogHeader className="flex-shrink-0">
          <DialogTitle>Claude Code 로그인</DialogTitle>
          <DialogDescription>
            터미널에서 Claude Code CLI와 상호작용하여 로그인을 완료하세요.
          </DialogDescription>
        </DialogHeader>
        <div className="flex-1 min-h-0">
          <ClaudeTerminal onClose={() => modal.remove()} />
        </div>
      </DialogContent>
    </Dialog>
  );
});
