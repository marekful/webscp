#!/bin/bash

TID="$1"
OWNPID=$$

#kill -SIGUSR1 $(pgrep -f -U agent "$TID" | awk '$1 !~ /'$OWNPID'/ {printf $1 " " }')

PIDS=$(pgrep -f -U agent "$TID" | awk '$1 !~ /'$OWNPID'/ {printf $1 " " }' | cut -d " " -f'1 2')

echo "PIDS: $PIDS"

kill -SIGUSR1 $PIDS
