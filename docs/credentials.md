# Credential storage

Hyprlume uses the session’s [Secret Service](https://specifications.freedesktop.org/secret-service/latest-single/) for application credentials. On Arch, install `gnome-keyring` if you need a provider. If another provider already owns `org.freedesktop.secrets`, Hyprlume uses that provider. Applications continue to store and read their secrets directly through the provider.

## Login and unlock

- For password-based desktop login, use your display manager’s existing GNOME Keyring PAM integration. For a custom login service, follow the [provider’s PAM instructions](https://wiki.gnome.org/Projects/GnomeKeyring/Pam/Manual) for `pam_gnome_keyring.so` in the login authentication and session stacks. The login keyring password must match the login password. Keep a recovery login available when changing a PAM stack.
- In **Settings → Credentials**, check the login and default collections. The two aliases can refer to the same collection. An unassigned alias is shown as such. Use **Start provider** for a provider registered for D-Bus activation.
- **Unlock…** delegates authentication to the provider’s own prompt, then reads the collection’s lock state. Use this after auto-login, a passwordless login, or when a collection was locked manually. **Lock** requests a collection lock through the same provider.
- Session locking dismisses an outstanding credential prompt. Collection lifetime is managed by the provider. On logout, applications should close their Secret Service connections as usual.

A missing collection can be created in the provider’s own keyring manager, such as Seahorse. Hyprlume reads only collection metadata, and its settings and agent protocol contain no application passwords. Hyprlume does not install or modify PAM configuration.
