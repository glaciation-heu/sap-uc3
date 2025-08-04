#!/bin/sh

program=$PROGRAM
party=$PARTY
host=${HOST:-localhost}

./compile.py $PROGRAM
./mascot-party.x $PARTY $PROGRAM -mp 11000 -N 2 -h $host
