#!/bin/bash

set -euo pipefail

ROOT="$(dirname "$(realpath "$0")")"

chmod() {
  /usr/bin/chmod "$@"
}

command() {
  if test -f /usr/bin/$1; then
    return 0
  elif test -f /usr/sbin/$1; then
    return 0
  else
    return 1
  fi
}

cp() {
  /usr/bin/cp "$@"
}

echo () {
  /usr/bin/echo "$1"
}

ln() {
  /usr/bin/ln "$@"
}

mkdir() {
  /usr/bin/mkdir -p "$@"
}

mv() {
  /usr/bin/mv "$@"
}

restorecon() {
  if command restorecon; then
    /usr/sbin/restorecon -rvF "$1"
  fi
}

sed() {
  /usr/bin/sed "$@"
}

sleep() {
  /usr/bin/sleep $1
}

systemctl() {
  /usr/bin/systemctl "$@"
}

startsWith() {
  case $2 in "$1"*) true;; *) false;; esac;
}

test() {
  /usr/bin/test "$@"
}

is_root() {
  if [ `/usr/bin/whoami` != 'root' ]; then
      echo "This script must be executed as root" 1>&2
      exit 100
  fi
}

is_x86_64() {
  if [[ $(/usr/bin/uname -m) != "x86_64" ]]; then
    echo "Currently, only x86_64 is supported. You need to build from source for other platforms."
    exit 1
  fi
}

is_debian() {
  # we detect by package manager, which is far more reliable than trying to
  # catch all existing distro names in a regex
  if test -f/usr/bin/apt; then
    echo "Detected Debian-based distro"
    return 0
  fi
  return 1
}

is_rhel() {
  # we detect by package manager, which is far more reliable than trying to
  # catch all existing distro names in a regex
  if test -f /usr/bin/dnf; then
    echo "Detected RHEL-based distro"
    return 0
  fi
  return 1
}

unknown_distro() {
  echo "Unknown / unsupported distro - currently supported: RHEL, Debian"
  exit 1
}

createConfig () {
  mkdir /etc/rauthy
  chmod 0600 /etc/rauthy

  if test -f /etc/rauthy/rauthy-pam-nss.toml ; then
    mv /etc/rauthy/rauthy-pam-nss.toml /etc/rauthy/rauthy-pam-nss.toml.$(date +%s)
  fi
  cp "$ROOT"/rauthy-pam-nss.toml /etc/rauthy/rauthy-pam-nss.toml
  chmod 0600 /etc/rauthy/rauthy-pam-nss.toml
  restorecon /etc/rauthy

  echo ""
  read -p "Rauthy URL (format: https://rauthy.example.com): " URL
  read -p "Host ID:     " ID
  read -p "Host Secret: " SECRET
  echo ""

  if startsWith http:// "$URL"; then
    echo "
CAUTION: You have given an insecure HTTP domain.
This MUST NEVER be used in production!
"
    while [ true ]; do
      read -p "Do you want to ignore this warning and use an insecure setup? (yes / no): " VALUE
      if [[ $VALUE == [yY][eE][sS] || $VALUE == [yY] ]]; then
        break
      elif [[ $VALUE == [nN][oO] || $VALUE == [n] ]]; then
        exit 1
      fi
    done
    sed -i "s/#danger_allow_insecure = false/danger_allow_insecure = true/g" /etc/rauthy/rauthy-pam-nss.toml
  elif ! startsWith https:// "$URL"; then
    echo "You have given invalid URL - missing scheme."
    exit 1
  fi

  URL=$(echo $URL | sed -e 's/\//\\\//g')
  sed -i "s/{{ rauthy_url }}/$URL/g" /etc/rauthy/rauthy-pam-nss.toml
  sed -i "s/{{ rauthy_host_id }}/$ID/g" /etc/rauthy/rauthy-pam-nss.toml
  sed -i "s/{{ rauthy_host_secret }}/$SECRET/g" /etc/rauthy/rauthy-pam-nss.toml

  mkdir /var/lib/pam_rauthy
  chmod 755 /var/lib/pam_rauthy
  cp "$ROOT"/session_scripts/session_* /var/lib/pam_rauthy/
  chmod 700 /var/lib/pam_rauthy/session_*
  restorecon /var/lib/pam_rauthy

  cp -r /etc/skel /etc/skel_rauthy

  echo "Created the config file /etc/rauthy/rauthy-pam-nss.toml ."
}

installAppArmor () {
  echo "TODO installAppArmor"
  exit 1
}

installSELinux() {
  if ! command getenforce; then
    echo "SELinux not found - skipping policy installation"
    return 0
  fi

  echo "Installing basic SELinux policies"

  # make sure it exists so it does not error
  mkdir /var/run/rauthy

  /usr/sbin/setsebool -P nis_enabled 1
  /usr/sbin/semodule -i "$ROOT"/selinux/rauthy-pam-nss.pp

  # The systemd_user_runtimedir_t type will typically not exist on servers without a graphical desktop
  if /usr/bin/seinfo -t systemd_user_runtimedir_t | grep systemd_user_runtimedir_t; then
    /usr/sbin/semodule -i "$ROOT"/selinux/rauthy-pam-desktop.pp
  fi

  restorecon /etc/rauthy
  restorecon /etc/skel_rauthy
  restorecon /usr/bin
  restorecon /usr/sbin
  restorecon /usr/local/bin
  restorecon /usr/local/sbin
  restorecon /var/run/rauthy
  restorecon /var/lib/pam_rauthy

  echo "SEModule named rauthy-pam-nss installed and nis_enabled boolean set"
}

installNSS () {
  echo "
Rauthy NSS setup

The /etc/nsswitch.conf is not applied via authselect and may be
temporary for testing. The install PAM step will persist it.
If your system is not using 'authselect' at all, this step
will already be persistent.
"

  echo "Stopping rauthy-nss service"
  systemctl stop rauthy-nss || echo 'rauthy-nss service not running'

  echo "Installing rauthy-nss service"
  ARCH=$(/usr/bin/uname -m)
  if [[ $ARCH == "x86_64" ]];then
    cp "$ROOT"/x86_64/rauthy-nss /usr/sbin/

    if is_rhel; then
      cp -f "$ROOT"/x86_64/libnss_rauthy.so.2 /lib64/libnss_rauthy.so.2
      if ! test -f /lib/libnss_rauthy.so.2; then
        ln -s /lib64/libnss_rauthy.so.2 /lib/libnss_rauthy.so.2
      fi
    elif is_debian; then
      cp -f "$ROOT"/x86_64/libnss_rauthy.so.2 /lib/x86_64-linux-gnu/libnss_rauthy.so.2
    else
      unknown_distro
    fi
  elif [[ $ARCH == "aarch64" || $ARCH == "arm64" ]]; then
    cp "$ROOT"/aarch64/rauthy-nss /usr/sbin/

    if is_rhel; then
      cp -f "$ROOT"/aarch64/libnss_rauthy.so.2 /lib64/libnss_rauthy.so.2
      if ! test -f /lib/libnss_rauthy.so.2; then
        ln -s /lib64/libnss_rauthy.so.2 /lib/libnss_rauthy.so.2
      fi
    elif is_debian; then
      # TODO don't have any debian aarch64 available, not 100% sure if the target is correct
      cp -f "$ROOT"/aarch64/libnss_rauthy.so.2 /lib/aarch64-linux-gnu/libnss_rauthy.so.2
    else
      unknown_distro
    fi
  else
    echo "Unsupported architecture"
    exit 1
  fi
  chmod 755 /usr/sbin/rauthy-nss
  restorecon /usr/sbin/rauthy-nss

  echo "Copying systemd service file into place"
  cp "$ROOT"/rauthy-nss.service /etc/systemd/system/rauthy-nss.service
  echo "Reloading daemon and enableing rauthy-nss.service"
  systemctl daemon-reload
  systemctl enable rauthy-nss --now

  echo "Creating nsswitch.conf backup and copying template"
  if ! test -f /etc/nsswitch.conf.bak-non-rauthy; then
    cp /etc/nsswitch.conf /etc/nsswitch.conf.bak-non-rauthy
  fi
  cp /etc/nsswitch.conf /etc/nsswitch.conf.$(date +%s)
  if is_rhel; then
    cp "$ROOT"/pam/rhel/nsswitch.conf /etc/nsswitch.conf
  elif is_debian; then
    cp "$ROOT"/pam/debian/nsswitch.conf /etc/nsswitch.conf
  else
    unknown_distro
  fi
  restorecon /etc/nsswitch.conf

  SUCCESS=false
  for i in {1..10}; do
    if grep 'This Host:' /var/log/rauthy/rauthy-nss.log > /dev/null; then
      SUCCESS=true
      break
    fi
    echo "Waiting for successful Host whoami ..."
    sleep 1
  done

  if $SUCCESS; then
    echo "
Rauthy NSS service installed.

Before you proceed with installing the PAM module and activating it,
make sure that NSS lookups are working without issues. You can for
instance use

> getent hosts

And you should see all hosts configured on Rauthy.
If you have any issues, you can check the logs:

> tail -f /var/log/rauthy/rauthy-nss.log
"
  else
    echo "
Rauthy NSS service installed, but no successful Host whoami after
10 retries. You either have an issue with your setup or have given
wrong config variables. You should NOT proceed installing PAM
without fixing this!

Check the service:
> systemctl status rauthy-nss

Check the config:
> cat /etc/rauthy/rauthy-pam-nss.toml

Check the logs:
> tail -f /var/log/rauthy/rauthy-nss.log

When everything is fine, you will see a Host whoami lookup after
the service restart. You then should also see the hosts configured
on Rauthy via something like

> getent hosts
"
    exit 1;
  fi

  echo "
When getent requests are working as expected, you can proceed with
the PAM module installation, but DO NOT try to install / use it
when NSS lookups are not working!
Additionally, BEFORE you install the PAM module, which can lock
you out in case of issues or misconfiguration, have at least one
session / terminal open that you don't use and only keep as a
backup, in case PAM does not work as expected! DO NOT use your
backup session for any testing!
"
}

installPAM () {
  if ! test -f /etc/rauthy/rauthy-pam-nss.toml ; then
    echo "
You must first execute the 'nss' install step and make sure
that NSS lookups are working as expected before installing the
PAM module.
"
    exit 1
  fi

  echo "
Rauthy PAM setup

This step will install the PAM module. Only proceed, if you
have executed the 'base' and 'nss' install steps beforehand
and when nss lookups like 'getent hosts' are working as
expected.

Make sure that you have a backup terminal / session open
in case you need to recover from a misconfiguration. DO NOT
use this backup session for the actual testing, as a bad PAM
configuration may lock you our of your system.
"

  while [ true ]; do
    read -p "Proceed with the PAM module installation now? (yes / no): " VALUE
    if [[ $VALUE == [yY][eE][sS] || $VALUE == [yY] ]]; then
      break
    elif [[ $VALUE == [nN][oO] || $VALUE == [n] ]]; then
      exit 1
    fi
  done

  while [ true ]; do
    read -p "Allow 'wheel-rauthy' members to use sudo? (yes / no): " VALUE
    if [[ $VALUE == [yY][eE][sS] || $VALUE == [yY] ]]; then
      if ! /usr/bin/grep ^%wheel-rauthy /etc/sudoers; then
        echo "
# Allow members of the wheel-rauthy group to use sudo
%wheel-rauthy ALL=(ALL) ALL
" >> /etc/sudoers
      fi
      break
    elif [[ $VALUE == [nN][oO] || $VALUE == [n] ]]; then
      break
    fi
  done

  ARCH=$(uname -m)
  if [[ $ARCH == "x86_64" ]];then

    if is_rhel; then

      cp "$ROOT"/x86_64/pam_rauthy.so /lib64/security/pam_rauthy.so
      chmod 755 /lib64/security/pam_rauthy.so
      restorecon /lib64/security/pam_rauthy.so

    elif is_debian; then

      cp "$ROOT"/x86_64/pam_rauthy.so /lib/x86_64-linux-gnu/security/pam_rauthy.so
      chmod 755 /lib/x86_64-linux-gnu/security/pam_rauthy.so

    else
      unknown_distro
    fi

  elif [[ $ARCH == "aarch64" || $ARCH == "arm64" ]]; then
    if is_rhel; then

      cp "$ROOT"/aarch64/pam_rauthy.so /lib64/security/pam_rauthy.so
      chmod 755 /lib64/security/pam_rauthy.so
      restorecon /lib64/security/pam_rauthy.so

    elif is_debian; then

      # TODO don't have any debian aarch64 available, not 100% sure if the target is correct
      cp "$ROOT"/aarch64/pam_rauthy.so /lib/aarch64-linux-gnu/security/pam_rauthy.so
      chmod 755 /lib/aarch64-linux-gnu/security/pam_rauthy.so

    else
      unknown_distro
    fi
  else
    echo "Unsupported architecture"
    exit 1
  fi

  if command authselect; then
    if ! test -f /etc/authselect/custom/rauthy/system-auth; then
      /usr/bin/authselect create-profile -b=local rauthy
    fi

    cp "$ROOT"/pam/rhel/system-auth /etc/authselect/custom/rauthy/
    cp "$ROOT"/pam/rhel/password-auth /etc/authselect/custom/rauthy/
    cp "$ROOT"/pam/rhel/nsswitch.conf /etc/authselect/custom/rauthy/

    echo "
Found 'authselect', assuming it is in use - created a custom profile in:
/etc/authselect/custom/rauthy/

This custom profile can be seen as a template. You should review the custom
profile and apply any custom settings you might have set right now, before
activating it.

Available profiles:
$(/usr/bin/authselect list)

Current profile + features:
$(/usr/bin/authselect current)

Note down which profile + features you are on right now, so you can change
back later on, if something goes wrong. After you have done that, the
activation is up to you with:

> authselect select custom/rauthy

If everything else was set up correctly, and you activated the new authselect
profile, PAM logins with Rauthy-managed accounts should be working. Test this
in detail to make sure everything is fine, but KEEP A BACKUP SESSION open. A
broken PAM setup can lock you out of your own system!
"
  else
    if is_rhel; then

      if ! test -f /etc/pam.d/system-auth.bak-non-rauthy; then
        cp /etc/pam.d/system-auth /etc/pam.d/system-auth.bak-non-rauthy
      fi
      if ! test -f /etc/pam.d/password-auth.bak-non-rauthy; then
        cp /etc/pam.d/password-auth /etc/pam.d/password-auth.bak-non-rauthy
      fi

      cp /etc/pam.d/system-auth /etc/pam.d/system-auth.$(date +%s)
      cp /etc/pam.d/password-auth /etc/pam.d/password-auth.$(date +%s)

      cp "$ROOT"/pam/rhel/system-auth /etc/pam.d/system-auth
      restorecon /etc/pam.d/system-auth
      cp "$ROOT"/pam/rhel/password-auth /etc/pam.d/password-auth
      restorecon /etc/pam.d/password-auth

    elif is_debian; then

      if ! test -f /etc/pam.d/common-auth.bak-non-rauthy; then
        cp /etc/pam.d/common-auth /etc/pam.d/common-auth.bak-non-rauthy
      fi
      if ! test -f /etc/pam.d/common-password.bak-non-rauthy; then
        cp /etc/pam.d/common-password /etc/pam.d/common-password.bak-non-rauthy
      fi
      if ! test -f /etc/pam.d/common-account.bak-non-rauthy; then
        cp /etc/pam.d/common-account /etc/pam.d/common-account.bak-non-rauthy
      fi
      if ! test -f /etc/pam.d/common-session.bak-non-rauthy; then
        cp /etc/pam.d/common-session /etc/pam.d/common-session.bak-non-rauthy
      fi

      cp /etc/pam.d/common-auth /etc/pam.d/common-auth.$(date +%s)
      cp /etc/pam.d/common-password /etc/pam.d/common-password.$(date +%s)
      cp /etc/pam.d/common-account /etc/pam.d/common-account.$(date +%s)
      cp /etc/pam.d/common-session /etc/pam.d/common-session.$(date +%s)

      cp "$ROOT"/pam/debian/common-auth /etc/pam.d/common-auth
      cp "$ROOT"/pam/debian/common-password /etc/pam.d/common-password
      cp "$ROOT"/pam/debian/common-account /etc/pam.d/common-account
      cp "$ROOT"/pam/debian/common-session /etc/pam.d/common-session

    else
      unknown_distro
    fi

    echo "
Backups of the existing rule were created and new files copied into /etc/pam.d/ .

If everything else was set up correctly, PAM logins with Rauthy-managed
accounts should be working now. Test this in detail to make sure everything
is fine, but KEEP A BACKUP SESSION open. A broken PAM setup can lock you
out of your own system!
    "
  fi
}

is_root
is_x86_64
if [ "$1" == 'nss' ]; then
  createConfig
  #installAppArmor
  installSELinux
  installNSS
elif [ "$1" == 'pam' ]; then
  installPAM
else
  echo "Unknown arg given, must be one of: nss pam"
  exit 1
fi

exit 0
