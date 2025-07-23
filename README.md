# Rauthy PAM / NSS Module

This crate is in a very experimental stage, and you should **NOT** try to use it in any serious context yet!

The goal of this project is to provide PAM and NSS modules for Rauthy to allow logging in to Servers and Workstations,
and basically anywhere else where PAM works, using Rauthy-managed hosts and accounts.

Non-exhaustive list of features:

- [x] Local Login with Password
- [x] Local login with Yubikey
- [x] NSS module to resolve non-local users (`getent passwd`) - `shadow` is currently implemented as well, but will
  probably be removed
    - [x] `getent passwd`
    - [x] `getent passwd <username>`
    - [x] `getent passwd <user_id>`
- [x] NSS module to resolve non-local groups (`getent group`)
    - [x] `getent group`
    - [x] `getent group <groupname>`
    - [x] `getent group <group_id>`
- [x] NSS module to resolve non-local hosts (`getent hosts`)
    - [x] `getent hosts`
    - [x] `getent hosts <hostname>` - Note: The module finds and returns the correct data, but `getent` e.g. does not
      display it for some reason. However, when I create a host named `batman` and then `ping batman`, even though
      `getent hosts batman` does not print the output, the `ping` command resolves it properly.
    - [x] `getent hosts <host_ip>`
- [x] Local Login with Password without local user -> resolved via NSS module
- [x] Local login with Yubikey without local user -> resolved via NSS module
- [x] `su - <rauthy_user>` with Password (on a local host)
- [x] `su - <rauthy_user>` with Yubikey (on a local host)
- [x] `su - <rauthy_user>` with Password (on a remote host)
- [ ] `su - <rauthy_user>` with Yubikey (on a remote host)
- [x] ssh into a host with a non-existent, Rauthy-managed user with Password
- [x] ssh into a host with a non-existent, Rauthy-managed user with online Passkey validation
- [x] `sudo` on remote host via SSH session - can be achieved by adding `%wheel-rauthy   ALL=(ALL)   ALL`
  to `/etc/sudoers`
- [ ] `su` on remote host via SSH session
- [ ] Login to window managers like `gdm` or `sddm`

On the Rauthy side, a lot of updates are necessary as well of course. I currently
have [Rauthy #1101](https://github.com/sebadob/rauthy/pull/1101) open to make it work in the end.

> While this project is even before its very first release, I will push directly to `main` for efficiency reasons and
> don't even care about PRs.
