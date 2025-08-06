set shell := ["bash", "-uc"]

export MSRV := `cat Cargo.toml | grep '^rust-version =' | cut -d " " -f3 | xargs`
export VERSION := `cat Cargo.toml | grep '^version =' | cut -d " " -f3 | xargs`
export USER := `echo "$(id -u):$(id -g)"`
builder_image := "ghcr.io/sebadob/rauthy-builder"
builder_tag_date := "20250804"
cargo_home := `echo ${CARGO_HOME:-$HOME/.cargo}`
container_cargo_registry := "/usr/local/cargo/registry"
docker := `echo ${DOCKER:-docker}`
map_docker_user := if docker == "podman" { "" } else { "-u $USER" }
container_image := "almalinux:10-kitten"
install_dir := "install/rauthy-pam-nss-install"
jemalloc_conf := "JEMALLOC_SYS_WITH_MALLOC_CONF=narenas:1"
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

    test -f install/rauthy-pam-nss-install.tar.gz && rm -f install/rauthy-pam-nss-install.tar.gz
    test -f {{ install_dir }}/LICENSE && rm -rf {{ install_dir }}
    mkdir -p {{ install_dir }}

    # x86_64
    {{ docker }} run \
        -v {{ cargo_home }}/registry:{{ container_cargo_registry }} \
        -v {{ invocation_directory() }}/:/work/ \
        -w /work \
        -e {{ jemalloc_conf }} \
        {{ map_docker_user }} \
        {{ builder_image }}:{{ builder_tag_date }} \
        cargo build --release --target x86_64-unknown-linux-gnu
    mkdir -p {{ install_dir }}/x86_64
    cp target/x86_64-unknown-linux-gnu/release/rauthy-nss {{ install_dir }}/x86_64/
    cp target/x86_64-unknown-linux-gnu/release/librauthy_pam.so {{ install_dir }}/x86_64/pam_rauthy.so
    cp target/x86_64-unknown-linux-gnu/release/librauthy_nss.so {{ install_dir }}/x86_64/libnss_rauthy.so.2

    # TODO the cross-compilation currently fails because of libudev
    # aarch64
    #{{ docker }} run \
    #    -v {{ cargo_home }}/registry:{{ container_cargo_registry }} \
    #    -v {{ invocation_directory() }}/:/work/ \
    #    -w /work \
    #    -e {{ jemalloc_conf }} \
    #    {{ map_docker_user }} \
    #    {{ builder_image }}:{{ builder_tag_date }} \
    #    cargo build --release --target aarch64-unknown-linux-gnu
    #mkdir -p {{ install_dir }}/aarch64
    #cp target/aarch64-unknown-linux-gnu/release/rauthy-nss {{ install_dir }}/aarch64/
    #cp target/aarch64-unknown-linux-gnu/release/librauthy_pam.so {{ install_dir }}/aarch64/pam_rauthy.so
    #cp target/aarch64-unknown-linux-gnu/release/librauthy_nss.so {{ install_dir }}/aarch64/libnss_rauthy.so.2

    # copy other files + templates
    cp LICENSE {{ install_dir }}/LICENSE
    echo $VERSION > {{ install_dir }}/VERSION

    cp install/install.sh {{ install_dir }}/install.sh
    cp install/rauthy-pam-nss.toml {{ install_dir }}/rauthy-pam-nss.toml

    cp -r templates/pam {{ install_dir }}/pam
    cp -r templates/session_scripts {{ install_dir }}/session_scripts
    cp templates/systemd/rauthy-nss.service {{ install_dir }}/rauthy-nss.service

    checkmodule -M -m -o selinux/rauthy-pam-nss.mod selinux/rauthy-pam-nss.te
    semodule_package -m selinux/rauthy-pam-nss.mod -o selinux/rauthy-pam-nss.pp
    mkdir {{ install_dir }}/selinux
    cp selinux/rauthy-pam-nss.* {{ install_dir }}/selinux

    tar -czf install/rauthy-pam-nss-install.tar.gz -C install ./rauthy-pam-nss-install
    git add install/rauthy-pam-nss-install.tar.gz

# build the SELinux module from selinux/ and apply it (ty == local / nis / nss / ssh)
apply-selinux ty="local":
    #!/usr/bin/env bash
    set -euxo pipefail

    cd selinux

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
