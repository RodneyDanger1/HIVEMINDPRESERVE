import { describe, expect, it } from 'vitest';
import { decodeTemplateCatalog } from '../src/state/templateCatalog';

function catalog(count: number, insertable: number) {
  return {
    templates: Array.from({ length: count }, (_, index) => ({
      template_id: `template-${index}`,
      name: `Template ${index + 1}`,
      inputs: [{ name: 'task', kind: 'string' }],
      insertable: index < insertable,
    })),
  };
}

describe('template gallery decoder', () => {
  it('shows all eight catalog templates but only four launchable ones', () => {
    const rows = decodeTemplateCatalog(catalog(8, 4));
    expect(rows).toHaveLength(8);
    expect(rows.filter((row) => row.insertable)).toHaveLength(4);
    expect(rows[0]).toMatchObject({
      id: 'template-0',
      label: 'Template 1',
      insertable: true,
      inputs: ['task'],
    });
    expect(rows[7].insertable).toBe(false);
  });

  it('rejects a payload with no templates array instead of inventing a catalog', () => {
    expect(() => decodeTemplateCatalog({})).toThrow(/no templates/);
    expect(() => decodeTemplateCatalog(null)).toThrow(/no templates/);
  });

  it('skips malformed rows and preserves missing input metadata as empty', () => {
    const rows = decodeTemplateCatalog({
      templates: [
        { template_id: 'ok', name: 'OK', insertable: true },
        { id: 'wrong-shape', name: 'Wrong' },
        null,
        42,
      ],
    });
    expect(rows).toEqual([
      { id: 'ok', label: 'OK', insertable: true, inputs: [] },
    ]);
  });
});
