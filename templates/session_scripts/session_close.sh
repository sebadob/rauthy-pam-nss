#/bin/bash

USERNAME=$1
USER_ID=$2
GROUP_ID=$3
RAUTHY_USER_ID=$4
RAUTHY_USER_EMAIL=$5

# just an example command
echo "$(date) $USER_ID:$GROUP_ID $USERNAME $RAUTHY_USER_ID/$RAUTHY_USER_EMAIL - session close" >> /tmp/rauthy_dbg
