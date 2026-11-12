import { describe, expect, it } from 'vitest';
import dataPanelSource from '../src/panels/DataPanel.tsx?raw';

describe('CAP-4 workbench mount', () => {
  it('mounts the real FileTree and Editor in the existing Data surface', () => {
    expect(dataPanelSource).toContain("from '../workbench'");
    expect(dataPanelSource).toContain('<FileTree');
    expect(dataPanelSource).toContain('<Editor');
    expect(dataPanelSource).toContain('buildFileTreeNodes(fileTree.entries)');
    expect(dataPanelSource).toContain('readOnly');
  });

  it('does not invent file rows or editable writes', () => {
    expect(dataPanelSource).not.toContain('sample');
    expect(dataPanelSource).not.toContain('mock');
    expect(dataPanelSource).not.toContain('onSave');
  });
});
