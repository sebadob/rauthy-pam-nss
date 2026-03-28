# show labels on a file

```bash
ls -lZ /var/run/rauthy/rauthy_proxy.sock
```

# temporary change labels for testing

```bash
chcon -t rauthy_var_run_t /var/run/rauthy/rauthy_proxy.sock
```

# restore labels

```bash
restorecon -v /var/run/rauthy/rauthy_proxy.sock
```

With `-R` to do it recursively.

# debug alerts

```bash
ausearch -c 'updatedb' --raw | audit2allow
```

# find all files with label

```bash
semanage fcontext -l | grep rauthy_etc_t
```

or via `sesearch`

```
sesearch (-A|-T|...) [-s SOURCE [-t TARGET [-c CLASS [-p PERMISSION]]]]
```
