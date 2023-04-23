#!/bin/bash

KEY_ID="$1"
KEY_FILE="/app/data/client/.ssh/id_ecdsa-$KEY_ID"
AUTHORIZED_KEYS_FILE="/app/data/client/.ssh/authorized_keys"

PKEY=$(cat "$KEY_FILE.pub") || exit 1

sed -i '\;^'"$PKEY"'$;d' "$AUTHORIZED_KEYS_FILE"

grep "$PKEY" "$AUTHORIZED_KEYS_FILE" && exit 2

rm -f "$KEY_FILE"
rm -f "$KEY_FILE-pem"
rm -f "$KEY_FILE.pub"
