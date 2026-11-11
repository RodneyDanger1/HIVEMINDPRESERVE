/** Decoder for the real `GET /api/templates` catalogue.
 *
 * Every catalogued template is shown in the library rail (the backend currently
 * ships eight); only `insertable: true` entries can be launched, because the
 * others have no insertable plan. This module is intentionally data-only so it
 * can be tested in the node test environment. */

export interface TemplateRow {
  readonly id: string;
  readonly label: string;
  readonly insertable: boolean;
  readonly inputs: readonly string[];
}

export function decodeTemplateCatalog(body: unknown): TemplateRow[] {
  const rawList = (body as { templates?: unknown } | null)?.templates;
  if (!Array.isArray(rawList)) {
    throw new Error('templates payload has no templates');
  }
  const decoded: TemplateRow[] = [];
  for (const raw of rawList) {
    if (typeof raw !== 'object' || raw === null) {
      continue;
    }
    const item = raw as Record<string, unknown>;
    if (typeof item.template_id !== 'string' || typeof item.name !== 'string') {
      continue;
    }
    const inputs = Array.isArray(item.inputs)
      ? item.inputs
          .map((input) =>
            typeof input === 'object' && input !== null
              ? (input as Record<string, unknown>).name
              : undefined,
          )
          .filter((name): name is string => typeof name === 'string')
      : [];
    decoded.push({
      id: item.template_id,
      label: item.name,
      insertable: item.insertable === true,
      inputs,
    });
  }
  return decoded;
}
