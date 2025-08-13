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

> This project is in a very early phase and even though many tests were done already, I expect some issues and rough
> edges to still exist, especially when it comes to SELinux policies.

## Install TL;DR

A more detailed documentation can be found in the [Rauthy Book](https://sebadob.github.io/rauthy/work/pam.html). The
tl;dr is:

```bash
curl -LO https://raw.githubusercontent.com/sebadob/rauthy-pam-nss/refs/heads/main/install/rauthy-pam-nss-install.tar.gz \
  && tar -xzf rauthy-pam-nss-install.tar.gz \
  && cd rauthy-pam-nss-install
```

Then, since you should never blindly execute a random bash script from the internet, especially with `sudo`, inspect
`install.sh` and afterward:

```bash
sudo ./install.sh nss
```

Then check via e.g. `getent hosts` or `getent groups` that you get data from Rauthy. However, the script does it as well
and you should see an error message about exceeded retries it you e.g. have given invalid credentials. When NSS lookups
are working fine, proceed with the PAM module installation:

```bash
sudo ./install.sh pam
```

If your OS is managed by `authselect`, you need to activate the new custom profile afterward with
`authselect select custom/rauthy`, just like mentioned in the output. On other OSes like Debian, the script will create
backups of config files and then copy the Rauthy configs in place directly.

**CAUTION:** Make sure to test all the logins and things that should work at this point BEFORE logging out. Keep a
backup session open, just in case something broke. Incorrectly configured PAM modules can lock you out of your machine!

## Limitations

Everything you need to do via SSH should be fine, as long as your configuration supports it. However, there is currently
one limitation regarding multiple chained `su -` via SSH. You simply cannot do this, when you use a Rauthy-managed
account. You can do a single one via e.g. `sudo su -` to become `root`, but you will not be able to do another `su -`
for a Rauthy-managed user from that session. You need to `exit` first to get to your root session. The reason is, that
the NSS module checks ENV vars from your session and depending on their values, it will either request a Remote PAM
Password from the account dashboard, or it will request your "real" password / Yubikey. If you have a password-only
account, this will work, but if your account is MFA secured, it's simply impossible to provide a USB Passkey via an
ssh remote connection.

Of course anyone can just modify their own env vars, but this is no security issue. If you mess up the `RAUTHY_*` env
vars, you will simply not be able to do anything authentication related anymore depending on your account setup.
