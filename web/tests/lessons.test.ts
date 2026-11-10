import { describe, expect, it } from 'vitest';
import { decodeLessons } from '../src/panels/LessonsPanel';

describe('authorized lesson projection', () => {
  it('decodes only complete lesson rows', () => {
    const rows = decodeLessons({
      lessons: [
        {
          proposal_id: 'p-1',
          area: 'governance',
          problem: 'votes are not persisted',
          proposal: 'persist four votes',
          state: 'applied_lesson',
        },
        { proposal_id: 'p-2', area: 'broken' },
      ],
    });
    expect(rows).toEqual([
      {
        proposalId: 'p-1',
        area: 'governance',
        problem: 'votes are not persisted',
        proposal: 'persist four votes',
      },
    ]);
  });

  it('returns empty for malformed bodies', () => {
    expect(decodeLessons(null)).toEqual([]);
    expect(decodeLessons({ lessons: 'no' })).toEqual([]);
  });
});
