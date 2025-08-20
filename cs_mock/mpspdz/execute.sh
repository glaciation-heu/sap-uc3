#!/bin/sh

program=$PROGRAM
party=$PARTY
host=${HOST:-localhost}
notification_port=734

while true; do
  echo "Waiting for the next computation job"
  nc -l -p $notification_port > /dev/null
  ./compile.py $PROGRAM
  # echo "Running offline phase"
  # ./mascot-party.x -mp 11000 -N 2 -h $host --offline-only $PARTY $PROGRAM
  # echo "Running online phase"
  if [ "$PROTOCOL" = "shamir" ]; then
    ./shamir-party.x -mp 11000 -N 2 -h $host $PARTY $PROGRAM
  else
    ./mascot-party.x -mp 11000 -N 2 -h $host $PARTY $PROGRAM
  fi
done
