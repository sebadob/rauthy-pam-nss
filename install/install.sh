#/bin/bash

set -euo pipefail

chmod() {
  /usr/bin/chmod "$@"
}

command() {
  /usr/bin/command -v $1 > /dev/null
}

cp() {
  /usr/bin/cp "$@"
}

echo () {
  /usr/bin/echo "$1"
}

mkdir() {
  /usr/bin/mkdir -p "$@"
}

mv() {
  /usr/bin/mv "$@"
}

#readYes() {
#  while [ true ]; do
#    /usr/bin/read -p "$1" VALUE
#    echo "value: $VALUE"
#    if [ $VALUE == [yY][eE][sS] || $VALUE == [yY] ]; then
#      return 0
#    elif [ $VALUE == [nN] || $VALUE == [nN][oO] ]; then
#      exit 1
#    fi
#  done
#}

sed () {
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

isRoot () {
  if [ `/usr/bin/whoami` != 'root' ]; then
      echo "This script must be executed as root" 1>&2
      exit 100
  fi
}

installTools () {
  echo 'Installing necessary tools'

  if command dnf; then
    # SELinux may not be installed
    if command getenforce; then
      # works on RHEL10, not tested on lower versions
      /usr/bin/dnf install checkpolicy setools-console
    fi
  elif command apt; then
    #/usr/bin/apt install blabla
    echo "Debian based distros have not been tested yet"
  else
    echo "Your distro has not been tested yet"
  fi
}

createConfig () {
  mkdir /etc/rauthy
  chmod 0600 /etc/rauthy

  if test -f /etc/rauthy/rauthy-pam-nss.toml ; then
    mv /etc/rauthy/rauthy-pam-nss.toml /etc/rauthy/rauthy-pam-nss.toml.$(date +%s)
  fi
  cp rauthy-pam-nss.toml /etc/rauthy/rauthy-pam-nss.toml
  chmod 0600 /etc/rauthy/rauthy-pam-nss.toml

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
  cp session_scripts/session_* /var/lib/pam_rauthy/
  chmod 700 /var/lib/pam_rauthy/session_*

  cp -r /etc/skel /etc/skel_rauthy

  echo "Created the config file /etc/rauthy/rauthy-pam-nss.toml ."
}

installAppArmor () {
  echo "TODO installAppArmor"
  exit 1
}

installSELinux() {
  if ! command -v getenforce; then
    echo "SELinux not found - skipping policy installation"
    return 0
  fi

  echo "Installing basic SELinux policies"

  /usr/sbin/setsebool -P nis_enabled 1

  /usr/bin/checkmodule -M -m -o selinux/rauthy-pam-nss.mod selinux/rauthy-pam-nss.te
  /usr/bin/semodule_package -m selinux/rauthy-pam-nss.mod -o selinux/rauthy-pam-nss.pp
  /usr/sbin/semodule -i selinux/rauthy-pam-nss.pp

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
  cp rauthy-nss /usr/local/sbin/
  chmod 755 /usr/local/sbin/rauthy-nss
  cp authselect/rauthy-nss.service /etc/systemd/system/rauthy-nss.service
  systemctl daemon-reload
  systemctl enable rauthy-nss --now

  echo "Creating nsswitch.conf backup and copying template"
  cp /etc/nsswitch.conf /etc/nsswitch.conf.$(date +%s)
  cp authselect/nsswitch.conf /etc/nsswitch.conf

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

If 'authselect' can be found, this setup is expecting that
it's used for managing your PAM setup. It will create a
custom profile for easy switching. If 'authselect' does not
exist, the files will be copied into /etc/pam.d/ manually
and backups of the existing ones will be created.
"

  while [ true ]; do
    read -p "Proceed with the PAM module installation now? (yes / no): " VALUE
    if [[ $VALUE == [yY][eE][sS] || $VALUE == [yY] ]]; then
      break
    elif [[ $VALUE == [nN][oO] || $VALUE == [n] ]]; then
      exit 1
    fi
  done

  echo "TODO finish PAM setup"

  exit 1
}

isRoot
if [ "$1" == 'nss' ]; then
  installTools
  createConfig
  # TODO
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
