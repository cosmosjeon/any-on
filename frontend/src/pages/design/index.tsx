import { useState } from 'react';
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Input } from '@/components/ui/input';
import { Send, Sparkles, Code, Eye } from 'lucide-react';
import { cn } from '@/lib/utils';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export function DesignPage() {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: '1',
      role: 'assistant',
      content: 'Hello! I\'m your AI design assistant. How can I help you build your UI today?',
      timestamp: new Date(),
    },
  ]);
  const [input, setInput] = useState('');
  const [previewMode, setPreviewMode] = useState<'code' | 'preview'>('preview');

  const handleSendMessage = () => {
    if (!input.trim()) return;

    const newMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input,
      timestamp: new Date(),
    };

    setMessages([...messages, newMessage]);
    setInput('');

    // Simulate AI response
    setTimeout(() => {
      const aiResponse: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'I\'ll help you with that! Let me generate the design...',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, aiResponse]);
    }, 1000);
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="border-b bg-background">
        <div className="flex items-center justify-between px-6 py-4">
          <div>
            <h1 className="text-2xl font-semibold">AI 디자인 어시스턴트</h1>
            <p className="text-sm text-muted-foreground mt-1">
              AI와 대화하며 UI 컴포넌트를 실시간으로 디자인하고 생성합니다.
            </p>
          </div>
        </div>
      </div>

      <ResizablePanelGroup direction="horizontal" className="flex-1">
        {/* Left Panel - Preview */}
        <ResizablePanel defaultSize={50} minSize={30}>
          <div className="h-full flex flex-col bg-muted/30">
            {/* Preview Header */}
            <div className="border-b bg-background px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Eye className="h-4 w-4 text-muted-foreground" />
                <h2 className="text-sm font-semibold">Preview</h2>
              </div>
              <div className="flex items-center gap-1">
                <Button
                  variant={previewMode === 'preview' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setPreviewMode('preview')}
                  className="h-8"
                >
                  <Eye className="h-3.5 w-3.5 mr-1.5" />
                  Preview
                </Button>
                <Button
                  variant={previewMode === 'code' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setPreviewMode('code')}
                  className="h-8"
                >
                  <Code className="h-3.5 w-3.5 mr-1.5" />
                  Code
                </Button>
              </div>
            </div>

            {/* Preview Content */}
            <ScrollArea className="flex-1">
              <div className="p-8">
                {previewMode === 'preview' ? (
                  <div className="bg-background rounded-lg border min-h-[600px] p-8">
                    <div className="flex flex-col items-center justify-center h-full text-center">
                      <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mb-4">
                        <Sparkles className="h-8 w-8 text-primary" />
                      </div>
                      <h3 className="text-xl font-semibold mb-2">
                        Start designing with AI
                      </h3>
                      <p className="text-muted-foreground max-w-md">
                        Describe what you want to build in the chat, and I'll generate
                        the UI components for you.
                      </p>
                    </div>
                  </div>
                ) : (
                  <div className="bg-background rounded-lg border p-4">
                    <pre className="text-sm">
                      <code>{`// Your generated code will appear here
export function Component() {
  return (
    <div>
      {/* AI-generated UI */}
    </div>
  );
}`}</code>
                    </pre>
                  </div>
                )}
              </div>
            </ScrollArea>
          </div>
        </ResizablePanel>

        <ResizableHandle withHandle />

        {/* Right Panel - Chat */}
        <ResizablePanel defaultSize={50} minSize={30}>
          <div className="h-full flex flex-col bg-background">
            {/* Chat Header */}
            <div className="border-b px-4 py-3">
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-primary" />
                <h2 className="text-sm font-semibold">AI Chat</h2>
              </div>
            </div>

            {/* Messages */}
            <ScrollArea className="flex-1">
              <div className="p-4 space-y-4">
                {messages.map((message) => (
                  <div
                    key={message.id}
                    className={cn(
                      'flex gap-3',
                      message.role === 'user' ? 'justify-end' : 'justify-start'
                    )}
                  >
                    {message.role === 'assistant' && (
                      <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                        <Sparkles className="h-4 w-4 text-primary" />
                      </div>
                    )}
                    <div
                      className={cn(
                        'rounded-lg px-4 py-2.5 max-w-[80%]',
                        message.role === 'user'
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted'
                      )}
                    >
                      <p className="text-sm whitespace-pre-wrap">{message.content}</p>
                      <span className="text-xs opacity-60 mt-1 block">
                        {message.timestamp.toLocaleTimeString([], {
                          hour: '2-digit',
                          minute: '2-digit',
                        })}
                      </span>
                    </div>
                    {message.role === 'user' && (
                      <div className="w-8 h-8 rounded-full bg-secondary flex items-center justify-center flex-shrink-0">
                        <span className="text-sm font-medium">You</span>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </ScrollArea>

            {/* Input Area */}
            <div className="border-t p-4">
              <div className="flex gap-2">
                <Input
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      handleSendMessage();
                    }
                  }}
                  placeholder="Describe what you want to build..."
                  className="flex-1"
                />
                <Button onClick={handleSendMessage} size="icon">
                  <Send className="h-4 w-4" />
                </Button>
              </div>
              <p className="text-xs text-muted-foreground mt-2">
                Press Enter to send, Shift+Enter for new line
              </p>
            </div>
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
