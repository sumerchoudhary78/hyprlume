# Removable drives

Install `udisks2` on Arch. Open **Desktop tools → Drives** or run `hyprlume storage`. The panel listens for UDisks2 events while open and shows USB and other removable drives. Use **Refresh** if the service has just been installed or restarted.

- **Mount** asks UDisks2 to mount a filesystem. **Open files** opens the observed mount point with the default directory handler through GIO.
- **Unlock…** passes the entered LUKS passphrase to UDisks2. You can then mount the cleartext filesystem. Unmount it before selecting **Lock**. The passphrase is cleared from the field on submission and is not saved in Settings or shown in receipts.
- **Safely remove…** asks for confirmation, unmounts the drive’s filesystems, locks its unlocked encrypted volumes, then requests eject or power-off. If a filesystem is busy, close the applications using it and retry. Wait for the successful removal receipt before unplugging the device.

System-managed devices and mounts outside the usual removable-media directories are protected. Power-off is unavailable for devices that share a UDisks2 sibling group. UDisks2 handles permissions and may display a Polkit authentication prompt.
