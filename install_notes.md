# WIP

This is a gist for installing the complete package on a fresh machine.

```bash
curl -LO http://192.168.14.20:8000/rauthy-pam-nss-install.tar.gz \
  && tar -xzf rauthy-pam-nss-install.tar.gz \
  && cd rauthy-pam-nss-install
```

Inspect `install.sh` to your liking, then

```bash
./install.sh nss
```

Then check via e.g. `getent hosts` or `getent groups` that you get data from Rauthy.

```bash
./install.sh pam
```

will install the PAM module. If your OS is managed by `authselect`, you need to activate the new custom profile
afterward.
