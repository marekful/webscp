#!/usr/bin/with-contenv bash

if grep ^ID=debian /etc/*release* > /dev/null; then
  exit 0
fi

echo "Alpine init"

mkdir -p /run/s6/container_environment /run/sshd

if grep agent /etc/passwd >/dev/null; then
  echo "User 'agent' exists"
else
  echo "Adding user 'agent'"
  if [ -z "$UUID" ]; then UUID=1000; fi
  adduser -h /app/data/client -s /bin/bash -D --uid "$UUID" agent
  sed -i s/agent:\!/"agent:*"/g /etc/shadow
fi

mkdir -p /app/data/temp
chown -R agent:agent /app/data

chown agent:agent /etc/scripts/uploader.sh /etc/scripts/cancel-transfer.sh \
                  /etc/scripts/generate-key-pair.sh /etc/scripts/revoke-key-pair.sh
chmod u+x /etc/scripts/uploader.sh /etc/scripts/cancel-transfer.sh \
          /etc/scripts/generate-key-pair.sh /etc/scripts/revoke-key-pair.sh

if [ -f /app/data/client/.ssh/id_rsa ];then
  echo "SSH keys exist"
  exit 0
fi

echo "Generating SSH keys"
mkdir -p /app/data/client/.ssh && \
    ssh-keygen -q -t rsa -b 4096 -f /app/data/client/.ssh/id_rsa -N "" && \
    chown -R agent:agent /app/data/client/.ssh && \
    mkdir -p /app/data/host-keys && \
    ssh-keygen -f /app/data/host-keys/ssh_host_ecdsa_key -N '' -t ecdsa