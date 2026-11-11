import { describe, expect, it } from 'vitest';
import { resolveProviderProfileForm } from '../src/api/providerProfile';

describe('provider profile form resolution', () => {
  it('prefers the saved profile fields when present', () => {
    const form = resolveProviderProfileForm({
      profile: {
        name: 'tokenlab',
        api_type: 'tokenlabs',
        base_url: 'https://lab.example/v1',
        wire_model: 'DeepSeek-V4.1-Flash',
        model: 'deepseek-v4-pro',
      },
      active: {
        api_type: 'tokenlabs',
        base_url: 'https://other.example/v1',
        wire_model: 'other',
        model: 'deepseek-flash',
      },
    });
    expect(form).toEqual({
      apiType: 'tokenlabs',
      baseUrl: 'https://lab.example/v1',
      wireModel: 'DeepSeek-V4.1-Flash',
      model: 'deepseek-v4-pro',
    });
  });

  it('falls back to the live configuration when the saved file omits base_url', () => {
    // install-desktop.ps1 writes name/api_type/model only for DeepSeek, so the
    // saved profile has no base_url while the server runs on the documented one.
    const form = resolveProviderProfileForm({
      profile: { name: 'deepseek', api_type: 'deepseek', model: 'deepseek-flash' },
      active: {
        api_type: 'deepseek',
        base_url: 'https://api.deepseek.com',
        wire_model: 'deepseek-flash',
        model: 'deepseek-flash',
      },
    });
    expect(form.baseUrl).toBe('https://api.deepseek.com');
    expect(form.wireModel).toBe('deepseek-flash');
    expect(form.apiType).toBe('deepseek');
  });

  it('uses defaults when there is no profile and no active configuration', () => {
    const form = resolveProviderProfileForm({});
    expect(form).toEqual({
      apiType: 'deepseek',
      baseUrl: '',
      wireModel: '',
      model: 'deepseek-flash',
    });
  });

  it('never renders whitespace-only fields as a value', () => {
    const form = resolveProviderProfileForm({
      profile: { api_type: '  ', base_url: '   ', model: '' },
      active: { api_type: 'tokenlabs', base_url: ' https://lab.example/v1 ', model: 'deepseek-v4-pro' },
    });
    expect(form.apiType).toBe('tokenlabs');
    expect(form.baseUrl).toBe('https://lab.example/v1');
    expect(form.model).toBe('deepseek-v4-pro');
  });

  it('treats an explicit null profile as "no saved file"', () => {
    const form = resolveProviderProfileForm({
      profile: null,
      active: { api_type: 'deepseek', base_url: 'https://api.deepseek.com' },
    });
    expect(form.apiType).toBe('deepseek');
    expect(form.baseUrl).toBe('https://api.deepseek.com');
  });
});
