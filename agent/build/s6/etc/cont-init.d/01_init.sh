#!/usr/bin/with-contenv bash

mkdir -p /run/s6/container_environment /run/sshd

if grep agent /etc/passwd >/dev/null; then
  echo "User 'agent' exists"
else
  echo "Adding user 'agent'"
  useradd -m agent -p $(echo "$AGENT_SECRET" | openssl passwd -1 -stdin)
fi

mkdir -p /home/agent/.tmp-data
chown -R agent:agent /home/agent

chown agent:agent /etc/scripts/uploader.sh /etc/scripts/cancel-transfer.sh
chmod u+x /etc/scripts/uploader.sh /etc/scripts/cancel-transfer.sh

if [ -f /home/agent/.ssh/id_rsa ];then
  echo "SSH keys exist"
  exit 0
fi

mkdir -p /home/agent/.ssh && \
    ssh-keygen -q -t rsa -b 4096 -f /home/agent/.ssh/id_rsa -N "" && \
    chown -R agent:agent /home/agent/.ssh