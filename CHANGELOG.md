# Changelog

## v0.2.0

### Changes

#### Install archive on releases page

The final build was removed from the repo and added to the releases page instead.

#### `aarch64` / `arm64` support

The install-archive now contains files for `aarch64` / `arm64` as well and it will not exit with an error on these
machines anymore.

#### Improved Health Checks

The NSS service does periodic health checks to not unnecessarily send out network request all the time, when the
configured Rauthy instance is down anyway. It does negative caching as well. However, this can become an issue in
unstable networks, because then you might have not working NSS lookups for 30 seconds in the worst case. For this
reason, the interval can now be configured, and the interval during unhealthy are 3 seconds by default. This will make
resources available again much sooner. The following new config variables can be set in
`/etc/rauthy/rauthy-pam-nss.toml`:

```toml
# You can execute custom scripts on session open / close.
# For the session open, it will be executed as the very last
# step, after the home dir was created. This can be used for
# instance to mount user home dirs via NFS or things like that.
#
# You cannot specify a complex command with options and must
# provide only the path to a script. Arguments will be
# automatically added in the following order:
#
# ./my_script.sh <username> <uid> <gid> <rauthy_user_id> <rauthy_user_email>
#
# CAUTION: These scripts will be executed as `root`!
# You MUST MAKE SURE that only `root` can modify them!
# -> `chmod 0700 path/to/script`
#
# NOTE: These scripts will only be executed during local login
# or sshd login, but NOT when you e.g. do an `su - <user>`.
#exec_session_open = '/var/lib/pam_rauthy/session_open.sh'
#exec_session_close = '/var/lib/pam_rauthy/session_close.sh'

# Define intervals for health checks. If a health check fails,
# NSS will not even try sending out requests until the status
# is back healthy to avoid excessive network requests during
# downtime.
#
# default: 30
health_check_interval_healthy = 30
#
# Note: If you have an unstable network, like e.g. Wifi or similar,
# you can decrease the unhealthy checks down to 1 second to reduce
# the time it takes until your machine can resolve users again.
# However, you avoid setting it to 0 on a heavily used machine.
#
# default: 3
health_check_interval_unhealthy = 3
```

#### ssh `AuthorizedKeysCommand`

This version, in combination with Rauthy v0.33, bringts support for the SSH `AuthorizedKeysCommand`.
A user can add public keys via the Account Dashboard for validation in addition to the already existing
PAM password. To make it work, you need to add the following lines to your `sshd_config`:

```
AuthorizedKeysCommand /usr/sbin/rauthy-authorized-keys
AuthorizedKeysCommandUser root
```

For more information on the configuration on Rauthys side, see
the [CHANGELOG](https://github.com/sebadob/rauthy/blob/main/CHANGELOG.md#authorizedkeys-for-pam-users).

#### Install Script updates

The install script has a new option:

`update`

If you already have an existing installation, you only want to execute

```
./install.sh update
```

This will keep your existing config untouched and only add the new values. It will also update SELinux policies (if
they exist in the system), the PAM and NSS modules and services, and it will install the new `rauthy-authorized-keys`.

### Bugfix

- The shebang in the install-script was not correctly set and would therefore error if executed from a non-BASH shell

## v0.1.0

Initial Release
