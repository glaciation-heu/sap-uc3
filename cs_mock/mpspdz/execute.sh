#!/bin/sh

program=$PROGRAM
party=$PARTY
host=${HOST:-localhost}
notification_port=734

while true; do
  echo "Waiting for the next computation job"
  nc -l -p $notification_port > /dev/null
  ./compile.py $PROGRAM
  ./mascot-party.x $PARTY $PROGRAM -mp 11000 -N 2 -h $host
done
