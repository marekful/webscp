#!/usr/bin/with-contenv bash

if grep ^ID=alpine /etc/*release* > /dev/null; then
  exit 0
fi

echo "Debian init"

mkdir -p /run/s6/container_environment /run/sshd

if grep agent /etc/passwd >/dev/null; then
  echo "User 'agent' exists"
else
  echo "Adding user 'agent'"
  useradd -m -s /bin/bash agent
  sed -i s/agent:\!/"agent:*"/g /etc/shadow
fi

mkdir -p /home/agent/.tmp-data
chown -R agent:agent /home/agent

chown agent:agent /etc/scripts/uploader.sh /etc/scripts/cancel-transfer.sh \
                  /etc/scripts/generate-key-pair.sh /etc/scripts/revoke-key-pair.sh
chmod u+x /etc/scripts/uploader.sh /etc/scripts/cancel-transfer.sh \
          /etc/scripts/generate-key-pair.sh /etc/scripts/revoke-key-pair.sh

if [ -f /home/agent/.ssh/id_rsa ];then
  echo "SSH keys exist"
  exit 0
fi

echo "Generating SSH keys"
mkdir -p /home/agent/.ssh && \
    ssh-keygen -q -t rsa -b 4096 -f /home/agent/.ssh/id_rsa -N "" && \
    chown -R agent:agent /home/agent/.ssh && \
    mkdir -p /etc/ssh/host-keys && \
    ssh-keygen -f /etc/ssh/host-keys/ssh_host_ecdsa_key -N '' -t ecdsa