#!/bin/bash

LOC=$1
ADR=$2
PRT=$3
REM=$4
CMD="wc -c < $REM"

PID=$$

size=$(wc -c < "$LOC")
scp -P "$PRT" "$LOC" "$ADR:$REM" | while read -t1; [[ $? -gt 128 ]] # loop while read times out, scp still running
do
    echo "stats::$(ssh -p "$PRT" "$ADR" "$CMD")/$size"
done