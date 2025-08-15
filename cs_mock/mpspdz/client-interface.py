#!/usr/bin/python3

import sys, random

#sys.path.insert(0, 'ExternalIO')
sys.path.append('.')

from client import *
from domains import *

base_port = 10000

client_id = 0

data = list(map(lambda x: int(x), sys.argv[1:]))
#bonus = [200, 300]

client = Client(['party0', 'party1'], base_port, client_id)


# 3. Send Private Inputs
# The client.send_private_inputs() function handles the entire
# secret sharing and communication protocol for you.
client.send_private_inputs(data)

# 4. Receive the Result
# The MPC program will reveal one value (the result of the comparison).
res_len = client.receive_outputs(1)
result = client.receive_outputs(res_len[0])
print(result)
