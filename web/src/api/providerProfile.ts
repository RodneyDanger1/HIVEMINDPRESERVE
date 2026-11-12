/**
 * Provider-profile form resolution.
 *
 * `GET /api/provider/profile` returns two things: the saved file (`profile`,
 * possibly absent) and the configuration the running server actually uses
 * (`active`). The saved file may legitimately omit fields — install-desktop.ps1
 * writes only name/api_type/model for DeepSeek — so the form must fall back to
 * `active` instead of showing an empty base URL that the server is not using.
 * Showing an empty field would misreport the live configuration and make Save
 * fail with E_SCHEMA_INVALID.
 */

export interface ProviderProfile {
  readonly name?: string;
  readonly api_type?: string;
  readonly base_url?: string;
  readonly model?: string;
  readonly wire_model?: string;
  readonly key_env?: string;
}

export interface ProviderProfileResponse {
  readonly profile?: ProviderProfile | null;
  readonly active?: ProviderProfile;
}

export interface ProviderProfileForm {
  readonly apiType: string;
  readonly baseUrl: string;
  readonly wireModel: string;
  readonly model: string;
}

const DEFAULT_API_TYPE = 'deepseek';
const DEFAULT_MODEL = 'deepseek-flash';

function text(value: string | undefined | null): string | null {
  if (typeof value !== 'string') {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length === 0 ? null : trimmed;
}

/** Field-by-field: saved profile first, then the live configuration, then the default. */
export function resolveProviderProfileForm(body: ProviderProfileResponse): ProviderProfileForm {
  const saved = body.profile ?? null;
  const active = body.active ?? null;
  return {
    apiType: text(saved?.api_type) ?? text(active?.api_type) ?? DEFAULT_API_TYPE,
    baseUrl: text(saved?.base_url) ?? text(active?.base_url) ?? '',
    wireModel: text(saved?.wire_model) ?? text(active?.wire_model) ?? '',
    model: text(saved?.model) ?? text(active?.model) ?? DEFAULT_MODEL,
  };
}
