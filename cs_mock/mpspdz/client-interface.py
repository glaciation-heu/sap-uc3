#!/usr/bin/python3

import sys, random
sys.path.append('.')

from client import *
from domains import *

base_port = 10000
client_id = 0


data = list(map(lambda x: int(x), sys.argv[1:]))
data_len = len(data)
# For now hardcoded party0 and party1, matches docker hostnames.
client = Client(['party0', 'party1'], base_port, client_id)

# Send Private Inputs
# First send len of data to all clients
for socket in client.sockets:
    os = octetStream()
    os.store(data_len)
    os.Send(socket)

# Now send all private inputs one by one
for x in data:
    client.send_private_inputs([x])

# Receive the Result
# First receive output len
res_len = client.receive_outputs(1)
# Than result
result = client.receive_outputs(res_len[0])
print(result)