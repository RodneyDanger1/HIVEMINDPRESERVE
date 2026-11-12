import { describe, expect, it } from 'vitest';
import agentsPanelSource from '../src/panels/AgentsPanel.tsx?raw';
import dataPanelSource from '../src/panels/DataPanel.tsx?raw';

describe('CAP-4 workbench mount', () => {
  it('mounts the real FileTree and Editor in the existing Data surface', () => {
    expect(dataPanelSource).toContain("from '../workbench'");
    expect(dataPanelSource).toContain('<FileTree');
    expect(dataPanelSource).toContain('<Editor');
    expect(dataPanelSource).toContain('buildFileTreeNodes(fileTree.entries)');
    expect(dataPanelSource).toContain('readOnly');
    expect(dataPanelSource).toContain('lg:grid-cols-[280px_1fr]');
    expect(dataPanelSource).toContain('grid-cols-1');
    // OLD: fixed grid-rows 40%/1fr clipped the Editor at 375px. NEW: the mobile
    // panel scrolls at natural row height; desktop keeps the fixed single row.
    expect(dataPanelSource).not.toContain('grid-rows-[minmax(0,40%)_minmax(0,1fr)]');
    expect(dataPanelSource).toContain('overflow-y-auto p-4 lg:grid-cols-[280px_1fr] lg:grid-rows-1');
  });

  it('mounts the real Timeline from server tool-call timestamps', () => {
    expect(agentsPanelSource).toContain('<Timeline');
    expect(agentsPanelSource).toContain('record.at ??');
    expect(agentsPanelSource).toContain('timelineSteps');
  });

  it('does not invent file rows or editable writes', () => {
    expect(dataPanelSource).not.toContain('sample');
    expect(dataPanelSource).not.toContain('mock');
    expect(dataPanelSource).not.toContain('onSave');
  });
});
