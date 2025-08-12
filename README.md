# Rauthy PAM / NSS Module

This project provides PAM and NSS modules for Rauthy to allow logging in to Servers and Workstations, and basically
anywhere else where PAM works, using Rauthy-managed hosts and accounts.

Supported features:

- [x] NSS module to resolve non-local users (`getent passwd`)
    - [x] `getent passwd`
    - [x] `getent passwd <username>`
    - [x] `getent passwd <user_id>`
- [x] NSS module to resolve non-local groups (`getent group`)
    - [x] `getent group`
    - [x] `getent group <groupname>`
    - [x] `getent group <group_id>`
    - [x] merged groups - Rauthy can manage groups with type `local` which it then will map to a locally
      existing `gid`, which again can be merged with proper config in `/etc/nsswitch.conf`
- [x] NSS module to resolve non-local hosts (`getent hosts`)
    - [x] `getent hosts`
    - [x] `getent hosts <hostname>` - Note: The module finds and returns the correct data, but `getent` e.g. does not
      display it for some reason. However, when I create a host named `batman` and then `ping batman`, even though
      `getent hosts batman` does not print the output, the `ping` command resolves it properly.
    - [x] `getent hosts <host_ip>`
- [x] Local Login with Password
- [x] Local login with Yubikey (or other USB Passkeys)
- [x] `su - <rauthy_user>` with Password (on a local host)
- [x] `su - <rauthy_user>` with Yubikey (on a local host)
- [x] `su - <rauthy_user>` on a remote host - works for both password and MFA accounts via PAM passwords from the
  account dashboard
- [x] ssh into a host with a non-existent, Rauthy-managed user with PAM Remote Password - both default password and
  MFA-secured accounts
- [x] `sudo` on remote host via SSH session - can be achieved by adding `%wheel-rauthy   ALL=(ALL)   ALL`
  to `/etc/sudoers`
- [x] Login to window managers like `gdm` or `sddm`
- [x] Copy custom `/etc/skel_rauthy` during home dir creation
- [x] optionally execute custom scripts on session open / close during login

> While this project is even before its very first release, I will push directly to `main` for efficiency reasons and
> don't even care about PRs.

## Install TL;DR

A more detailed documentation can be found in the Rauthy Book. The tl;dr is:

```bash
curl -LO https://raw.githubusercontent.com/sebadob/rauthy-pam-nss/refs/heads/main/install/rauthy-pam-nss-install.tar.gz \
  && tar -xzf rauthy-pam-nss-install.tar.gz \
  && cd rauthy-pam-nss-install
```

Then, since you should never blindly execute a random bash script from the internet, inspect `isntall.sh` and afterward:

```bash
sudo ./install.sh nss
```

Then check via e.g. `getent hosts` or `getent groups` that you get data from Rauthy. However, the script does it as well
and you should see an error message about exceeded retries it you e.g. have given invalid credentials. When NSS lookups
are working fine, proceed with the PAM module installation:

```bash
sudo ./install.sh pam
```

If your OS is managed by `authselect`, you need to activate the new custom profile afterward, just like mentioned in the
output. On other OSes like Debian, the script will create backups of config files and then copy the Rauthy configs in
place directly.

**CAUTION:** Make sure to test all the logins and things that should work at this point BEFORE logging out. Keep a
backup session open, just in case something broke. Incorrectly configured PAM modules can lock you out of your machine!
