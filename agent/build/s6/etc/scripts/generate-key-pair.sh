#!/bin/bash

KEY_ID="$1"
USER_ID="$2"
USER_NAME="$3"
INSTANCE_NAME="$4"

SSH_DIR="/app/data/client/.ssh"
KEY_FILE="$SSH_DIR/id_ecdsa-$KEY_ID"
AUTHORIZED_KEYS_FILE="$SSH_DIR/authorized_keys"

ssh-keygen -t ecdsa -b 256 -N "" -f "$KEY_FILE" >/dev/null || exit 1

cp "$KEY_FILE" "$KEY_FILE-pem" >/dev/null || exit 2

ssh-keygen -p -m pem -N "" -f "$KEY_FILE-pem" >/dev/null  || exit 3

cat "$KEY_FILE.pub" >> "$AUTHORIZED_KEYS_FILE" || exit 4

PEM_KEY=$(cat "$KEY_FILE-pem" | sed '1d;$d' | tr -d '\n')

HASH_FILE="$(echo -n "$PEM_KEY" | openssl sha256 | awk '{print $2}')"

echo "{\"id\": $USER_ID, \"name\": \"$USER_NAME\", \"branding\": \"$INSTANCE_NAME\"}" > "$SSH_DIR/$HASH_FILE"

#cat "$KEY_FILE-pem" | sed '1d;$d' | tr -d '\n'

echo -n "$PEM_KEY"
