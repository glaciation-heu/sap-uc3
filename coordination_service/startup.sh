#!/bin/sh

mkdir -p ~/.cs

# Create nework alias for csmock
echo "ephemeral-generic.default.csmock csmock" > /etc/host.aliases
export HOSTALIASES=/etc/host.aliases

/usr/local/bin/coordination_service
