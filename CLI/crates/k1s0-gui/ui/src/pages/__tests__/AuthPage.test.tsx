import { beforeEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import AuthPage from '../AuthPage';

const defaults = {
  discovery_url: 'https://issuer.example.com/.well-known/openid-configuration',
  client_id: 'k1s0-gui',
  scope: 'openid profile email',
};

beforeEach(() => {
  mockInvoke.mockReset();
  window.localStorage.clear();
});

describe('AuthPage', () => {
  it('loads device authorization defaults into the form', async () => {
    mockInvoke.mockResolvedValueOnce(defaults);

    renderWithProviders(<AuthPage />);

    expect(await screen.findByDisplayValue(defaults.discovery_url)).toBeInTheDocument();
    expect(screen.getByDisplayValue(defaults.client_id)).toBeInTheDocument();
    expect(screen.getByDisplayValue(defaults.scope)).toBeInTheDocument();
  });

  it('shows the connection failure in the GUI', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValueOnce(defaults).mockRejectedValueOnce('DNS lookup failed');

    renderWithProviders(<AuthPage />);

    await screen.findByDisplayValue(defaults.discovery_url);
    await user.click(screen.getByTestId('btn-check-connection'));

    expect(await screen.findByTestId('connection-message')).toHaveTextContent(
      `Failed to resolve ${defaults.discovery_url}. DNS lookup failed`,
    );
  });

  it('starts the device flow with the edited OIDC settings', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(defaults)
      .mockResolvedValueOnce({
        issuer: 'https://issuer.example.com',
        client_id: 'desktop-gui',
        scope: defaults.scope,
        token_endpoint: 'https://issuer.example.com/token',
        device_code: 'device-code',
        user_code: 'ABCD-EFGH',
        verification_uri: 'https://issuer.example.com/verify',
        verification_uri_complete: 'https://issuer.example.com/verify?user_code=ABCD-EFGH',
        interval: 5,
        expires_in: 600,
      });

    renderWithProviders(<AuthPage />);

    await screen.findByDisplayValue(defaults.discovery_url);
    await user.clear(screen.getByTestId('input-client-id'));
    await user.type(screen.getByTestId('input-client-id'), 'desktop-gui');
    await user.click(screen.getByTestId('btn-start-auth'));

    expect(await screen.findByText('ABCD-EFGH')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'start_device_authorization', {
      settings: {
        ...defaults,
        client_id: 'desktop-gui',
      },
    });
  });
});
