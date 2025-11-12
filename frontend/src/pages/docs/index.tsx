import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Plus } from 'lucide-react';

interface Document {
  id: string;
  title: string;
  description: string;
  createdAt: Date;
}

interface DocumentCategory {
  id: string;
  title: string;
  documents: Document[];
}

const initialCategories: DocumentCategory[] = [
  {
    id: 'planning',
    title: '기획',
    documents: [
      {
        id: 'prd-1',
        title: 'Product Requirements Document',
        description: '핵심 기능, 사용자 스토리, 기술 요구사항을 정의합니다.',
        createdAt: new Date('2024-01-15'),
      },
      {
        id: 'user-flow-1',
        title: 'User Flow Document',
        description: '사용자 경험과 흐름을 시각화하고 설명합니다.',
        createdAt: new Date('2024-01-20'),
      },
    ],
  },
  {
    id: 'design',
    title: '디자인',
    documents: [
      {
        id: 'design-system-1',
        title: 'Design System',
        description: '컬러 팔레트, 타이포그래피, 재사용 가능한 컴포넌트를 정의합니다.',
        createdAt: new Date('2024-01-18'),
      },
    ],
  },
  {
    id: 'technical',
    title: '기술',
    documents: [
      {
        id: 'tech-design-1',
        title: 'Technical Design Document',
        description: '시스템 아키텍처, 데이터베이스 스키마, API 엔드포인트를 상세히 설명합니다.',
        createdAt: new Date('2024-01-22'),
      },
    ],
  },
];

export function DocsPage() {
  const [categories] = useState<DocumentCategory[]>(initialCategories);

  const handleAddDocument = (categoryId: string) => {
    // TODO: 문서 추가 다이얼로그 열기
    console.log('Add document to category:', categoryId);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b bg-background">
        <div className="flex items-center justify-between px-6 py-4">
          <div>
            <h1 className="text-2xl font-semibold">프로젝트 문서</h1>
            <p className="text-sm text-muted-foreground mt-1">
              프로젝트의 핵심 문서들을 영역별로 관리합니다.
            </p>
          </div>
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="p-6">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {categories.map((category) => (
              <div key={category.id} className="flex flex-col bg-muted/30 rounded-lg p-4 border">
                {/* Category Header */}
                <div className="mb-4">
                  <div className="flex items-center justify-between mb-3">
                    <h2 className="text-lg font-semibold">{category.title}</h2>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleAddDocument(category.id)}
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                </div>

                {/* Documents List */}
                <div className="space-y-3">
                  {category.documents.length === 0 ? (
                    <Card className="bg-background/50">
                      <CardContent className="pt-6 pb-6">
                        <div className="text-center text-sm text-muted-foreground">
                          <p>문서가 없습니다</p>
                          <Button
                            size="sm"
                            variant="link"
                            className="mt-2"
                            onClick={() => handleAddDocument(category.id)}
                          >
                            첫 문서 추가하기
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  ) : (
                    category.documents.map((doc) => (
                      <Card
                        key={doc.id}
                        className="cursor-pointer hover:shadow-md transition-shadow hover:border-primary/50 bg-background"
                      >
                        <CardHeader className="p-4">
                          <div className="flex flex-col gap-1">
                            <CardTitle className="text-base leading-tight">
                              {doc.title}
                            </CardTitle>
                            <CardDescription className="text-xs line-clamp-2">
                              {doc.description}
                            </CardDescription>
                            <p className="text-xs text-muted-foreground mt-2">
                              {doc.createdAt.toLocaleDateString('ko-KR')}
                            </p>
                          </div>
                        </CardHeader>
                      </Card>
                    ))
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
