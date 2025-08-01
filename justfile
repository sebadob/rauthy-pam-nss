set shell := ["bash", "-uc"]

export MSRV := `cat Cargo.toml | grep '^rust-version =' | cut -d " " -f3 | xargs`
export VERSION := `cat Cargo.toml | grep '^version =' | cut -d " " -f3 | xargs`
export USER := `echo "$(id -u):$(id -g)"`
docker := `echo ${DOCKER:-docker}`
map_docker_user := if docker == "podman" { "" } else { "-u $USER" }
container_image := "almalinux:10-kitten"
install_dir := "install/rauthy-pam-nss-install"
pam_file := "rauthy-test"
test_user := "sebadob"

[private]
default:
    @just -l

# prints out the currently set version
version:
    #!/usr/bin/env bash
    echo "v$TAG"

# clippy lint + check with minimal versions from nightly
check:
    #!/usr/bin/env bash
    set -euxo pipefail
    clear
    cargo update
    cargo clippy -- -D warnings
    cargo minimal-versions check

    # update at the end again for following clippy and testing
    cargo update

# checks all combinations of features with clippy
clippy:
    #!/usr/bin/env bash
    set -euxo pipefail
    clear
    cargo clippy

# builds the nss proxy
build profile="debug":
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo build {{ profile }}

# builds the nss proxy in release mode and copies it into the system
build-install:
    #!/usr/bin/env bash
    set -euxo pipefail

    sudo systemctl stop rauthy-nss || echo 'rauthy-nss service not running'

    cargo build --release

    sudo cp target/release/rauthy-nss /usr/local/sbin/

    sudo cp templates/authselect/rauthy-nss.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl start rauthy-nss
    sudo systemctl status rauthy-nss

# does everything necessary to build the install/rauthy-pam-nss-install dir
build-install-archive:
    #!/usr/bin/env bash
    set -euxo pipefail

    test -f {{ install_dir }}/LICENSE && rm -rf {{ install_dir }}
    mkdir -p {{ install_dir }}

    # TODO should be done inside a container with lower ldd version
    cargo build --release
    cp target/release/rauthy-nss install/rauthy-pam-nss-install/
    cp target/release/librauthy_pam.so install/rauthy-pam-nss-install/pam_rauthy.so
    cp target/release/librauthy_nss.so install/rauthy-pam-nss-install/libnss_rauthy.so.2

    cp LICENSE {{ install_dir }}/LICENSE
    echo $VERSION > {{ install_dir }}/VERSION

    cp install.sh {{ install_dir }}/install.sh
    cp rauthy-pam-nss.toml {{ install_dir }}/rauthy-pam-nss.toml

    cp -r templates/authselect {{ install_dir }}/authselect
    cp -r templates/session_scripts {{ install_dir }}/session_scripts
    cp -r selinux {{ install_dir }}/selinux

# build the SELinux module from selinux/ and apply it (ty == local / nis / nss / ssh)
apply-selinux ty="local":
    #!/usr/bin/env bash
    set -euxo pipefail

    cd selinux

    checkmodule -M -m -o rauthy-pam-nss.mod rauthy-pam-nss.te && \
    semodule_package -m rauthy-pam-nss.mod -o rauthy-pam-nss.pp && \
    semodule -i rauthy-pam-nss.pp

    if [[ {{ ty }} == "local" ]]; then
        echo 'Building and applying SELinux rules for local login'
        checkmodule -M -m -o pam-rauthy-local.mod pam-rauthy-local.te && \
        semodule_package -m pam-rauthy-local.mod -o pam-rauthy-local.pp && \
        sudo semodule -i pam-rauthy-local.pp
    elif [[ {{ ty }} == "gdm" ]]; then
        echo 'Building and applying SELinux rules for gdm login'
        checkmodule -M -m -o pam-rauthy-gdm.mod pam-rauthy-gdm.te && \
        semodule_package -m pam-rauthy-gdm.mod -o pam-rauthy-gdm.pp && \
        sudo semodule -i pam-rauthy-gdm.pp
    elif [[ {{ ty }} == "kvm" ]]; then
        echo 'Building and applying SELinux rules for kvm'
        checkmodule -M -m -o nss-rauthy-kvm.mod nss-rauthy-kvm.te && \
        semodule_package -m nss-rauthy-kvm.mod -o nss-rauthy-kvm.pp && \
        sudo semodule -i nss-rauthy-kvm.pp
    elif [[ {{ ty }} == "nis" ]]; then
        setsebool -P nis_enabled 1
    elif [[ {{ ty }} == "nss" ]]; then
        echo 'Building and applying SELinux rules for NSS lookups'
        checkmodule -M -m -o rauthy-nss-uds-access.mod rauthy-nss-uds-access.te && \
        semodule_package -m rauthy-nss-uds-access.mod -o rauthy-nss-uds-access.pp && \
        sudo semodule -i rauthy-nss-uds-access.pp
    elif [[ {{ ty }} == "script" ]]; then
        echo 'Building and applying SELinux rules for skel dir copy'
        checkmodule -M -m -o pam-rauthy-script.mod pam-rauthy-script.te && \
        semodule_package -m pam-rauthy-script.mod -o pam-rauthy-script.pp && \
        sudo semodule -i pam-rauthy-script.pp
    elif [[ {{ ty }} == "skel" ]]; then
        echo 'Building and applying SELinux rules for skel dir copy'
        checkmodule -M -m -o pam-rauthy-skel.mod pam-rauthy-skel.te && \
        semodule_package -m pam-rauthy-skel.mod -o pam-rauthy-skel.pp && \
        sudo semodule -i pam-rauthy-skel.pp
    elif [[ {{ ty }} == "ssh" ]]; then
        echo 'Building and applying SELinux rules for ssh login'
        checkmodule -M -m -o pam-rauthy-ssh.mod pam-rauthy-ssh.te && \
        semodule_package -m pam-rauthy-ssh.mod -o pam-rauthy-ssh.pp && \
        sudo semodule -i pam-rauthy-ssh.pp
    fi

# remove the SELinux modules
remove-selinux:
    #!/usr/bin/env bash
    set -euxo pipefail
    sudo semodule -r pam-rauthy-local
    sudo semodule -r pam-rauthy-gdm
    sudo semodule -r pam-rauthy-nss
    sudo semodule -r pam-rauthy-script
    sudo semodule -r pam-rauthy-skel
    sudo semodule -r pam-rauthy-ssh
    setsebool -P nis_enabled 0

# create release build and copy it into /usr/lib64/security/pam_rauthy.so
install-pam:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --release

    # Remove either an existing file or symlink
    test -f /usr/lib64/security/pam_rauthy.so && sudo rm -f /usr/lib64/security/pam_rauthy.so
    sudo cp target/release/librauthy_pam.so /usr/lib64/security/pam_rauthy.so

# build an install the nss module
install-nss:
    #!/usr/bin/env bash
     set -euxo pipefail

     cargo build --release

     test -f /usr/lib64/libnss_r && sudo rm /usr/lib64/libnss_r
     test -f /usr/lib64/libnss_rauthy.so.2 && sudo rm /usr/lib64/libnss_rauthy.so.2

     sudo cp target/release/librauthy_nss.so /usr/lib64/libnss_rauthy.so.2
     sudo cp target/release/librauthy_nss.so /lib/libnss_rauthy.so.2

# copies templates/authselect/ to /etc/authselect/custom/rauthy/system-auth and re-applies it
update-authselect:
    #!/usr/bin/env bash
    set -euxo pipefail

    # Expects an already created custom authselect profile named `rauthy`

    sudo cp templates/authselect/system-auth /etc/authselect/custom/rauthy/system-auth
    sudo cp templates/authselect/password-auth /etc/authselect/custom/rauthy/password-auth
    sudo cp templates/authselect/nsswitch.conf /etc/authselect/custom/rauthy/nsswitch.conf
    sudo authselect select custom/rauthy

# run the code using `pamtester`
run ty="auth": install-pam
    #!/usr/bin/env bash
    set -euxo pipefail
    clear

    #cargo build
    if [[ {{ ty }} == "auth" ]]; then
      sudo pamtester {{ pam_file }} {{ test_user }} authenticate
    elif [[ {{ ty }} == "account" ]]; then
      sudo pamtester {{ pam_file }} {{ test_user }} acct_mgmt
    elif [[ {{ ty }} == "session_open" ]]; then
      sudo pamtester {{ pam_file }} {{ test_user }} open_session
    elif [[ {{ ty }} == "session_close" ]]; then
      sudo pamtester {{ pam_file }} {{ test_user }} close_session
    elif [[ {{ ty }} == "authtok" ]]; then
      sudo pamtester {{ pam_file }} {{ test_user }} chauthtok
    fi

# verifies the MSRV
msrv-verify:
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo msrv verify

# find's the new MSRV, if it needs a bump
msrv-find:
    cargo msrv find --min {{ MSRV }}

# verify thats everything is good
verify:
    # we don't want to rebuild the UI each time because it's checked into git
    #just build ui
    just check
    just clippy
    just msrv-verify

# makes sure everything is fine
verify-is-clean: verify
    #!/usr/bin/env bash
    set -euxo pipefail

    # make sure everything has been committed
    git diff --exit-code

    echo all good

# sets a new git tag and pushes it
release:
    #!/usr/bin/env bash
    set -euxo pipefail

    # make sure git is clean
    git diff --quiet || exit 1

    git tag "v$TAG"
    git push origin "v$TAG"

    just build-image
