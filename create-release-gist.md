# Create Release Archive

Creating a complete release archive is a bit more involved right now. We need to make sure to put the correct binary
hashes into the install file, modify custom update steps inside it, and we need to build the arm version on a different
machine and copy it over. Cross-compilation does not work right.

# Base System Updates

1. Check `Cargo.toml` for any manual upgrades
2. `just update`
3. Make sure to set the version number correctly

# Build `aarch64`

Build the `aarch64` version on a different machine. Make sure the repo is up to date.

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

Then copy the `aarch64` binaries into `install/aarch64/`.

Here is a little helper script. Needs to be adjusted for your system:

```bash
#!/bin/bash

set -euo pipefail

SSH_KEY_FILE="id_ed25519"
SERVER="opc@152.70.161.53"

TARGET_LOCAL="/home/sd/work/rauthy-pam-nss/install/aarch64"
TARGET_REMOTE="/home/opc/rauthy-pam-nss/target/aarch64-unknown-linux-gnu/release"

mkdir -p $TARGET_LOCAL
rm -f $TARGET_LOCAL/*

scp -i $SSH_KEY_FILE $SERVER:$TARGET_REMOTE/librauthy_pam.so $TARGET_LOCAL/pam_rauthy.so
scp -i $SSH_KEY_FILE $SERVER:$TARGET_REMOTE/librauthy_nss.so $TARGET_LOCAL/libnss_rauthy.so.2
scp -i $SSH_KEY_FILE $SERVER:$TARGET_REMOTE/rauthy-nss $TARGET_LOCAL/rauthy-nss
scp -i $SSH_KEY_FILE $SERVER:$TARGET_REMOTE/rauthy-authorized-keys $TARGET_LOCAL/rauthy-authorized-keys
```

# Build `x86_64`

```bash
just build-install-archive
```

# Grab Binary Hashes

We need to grab the SHA hashes of the binaries, and put them into `install/install.sh`.

```bash
sha256sum install/rauthy-pam-nss-install/aarch64/pam_rauthy.so &&
sha256sum install/rauthy-pam-nss-install/x86_64/pam_rauthy.so
```

Then create a new variable at the top of the file and handle it in `isInstalledVersion ()` and `update ()`.

# Final Build

After the SHA hashes have been grabbed and the install file was updated, build it once again with the new install
script.

```bash
just build-install-archive
```

The final files are:

```bash
install/rauthy-pam-nss-install.tar.gz
install/rauthy-pam-nss-install.tar.gz.sha256
```

Make sure to test both of these on a RHEL and Debian-based distro. Fresh install and upgrade from an earlier version.

```bash
sha256sum -c rauthy-pam-nss-install.tar.gz.sha256 && \
    tar -xzf rauthy-pam-nss-install.tar.gz && \
    cd rauthy-pam-nss-install
```

Update from an earlier version:

```bash
sudo ./install.sh update
```

Fresh install:

```bash
sudo ./install.sh nss && sudo ./install.sh pam
```

# Release

Only when the full test was fine, you can use these files for the final release.
