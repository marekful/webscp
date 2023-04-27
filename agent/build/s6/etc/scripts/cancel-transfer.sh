#!/bin/bash

TID="$1"
OWNPID=$$

if [ "$DISTRO" = "debian" ]; then
    PIDS=$(pgrep -f -U agent "$TID" | awk '$1 !~ /'$OWNPID'/ {printf $1 " " }' | cut -d " " -f'1 2')
elif [ "$DISTRO" = "alpine" ]; then
    PIDS=$(pgrep -f -U agent "$TID" | awk '$1 !~ /'$OWNPID'/ {printf $1 " " }')
fi
echo "PIDS: $PIDS"

kill -SIGUSR1 $PIDS
