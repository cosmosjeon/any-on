import { describe, it, expect, vi, beforeEach } from 'vitest';
import { projectsApi } from '../api';

// Mock fetch globally
globalThis.fetch = vi.fn() as typeof fetch;

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('projectsApi', () => {
    it('should fetch all projects', async () => {
      const mockProjects = [
        { id: '1', name: 'Test Project 1', path: '/test1' },
        { id: '2', name: 'Test Project 2', path: '/test2' },
      ];

      (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockProjects,
      } as Response);

      const result = await projectsApi.getAll();

      expect(globalThis.fetch).toHaveBeenCalledWith('/api/projects', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result).toEqual(mockProjects);
    });

    it('should handle API errors', async () => {
      (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
      } as Response);

      await expect(projectsApi.getAll()).rejects.toThrow();
    });
  });
});
