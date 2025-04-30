#!/bin/bash

protocol=$1 #e.g., mascot
program=$2 #e.g., tutorial, pp-poc
party_number=$3
nparties=$4
status_file=$5
server_ip=$6

params=""
if [ $party_number -ne 0 ]; then
  params="-h $server_ip"
fi
if [ "$protocol" != "yao" -a "$protocol" != "sy-rep-ring" ]; then
  params="$params -N $nparties"
fi

echo "Starting MP-SPDZ with protocol '$protocol' and params '$params' for '$nparties' parties on program '$program'"
./${protocol}-party.x -p $party_number $params  -v $program > $status_file 2>&1 #write stderr and stdout to file #-v for verbose
touch "DONE-$status_file" #computation finished (see api-srv/src/api.ts getDoneFile())