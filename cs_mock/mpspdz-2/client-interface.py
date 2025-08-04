#!/usr/bin/python3

import sys, random

#sys.path.insert(0, 'ExternalIO')
sys.path.append('.')

print("Running Client")

from client import *
from domains import *
print("After import")

base_port = 10000
print("Creating client")

client_id = 0
nr_of_outputs = int(sys.argv[1])
#bonus = list(map(lambda x: int(x), sys.argv[2:]))
bonus = [200, 300]

client = Client(['party0', 'party1'], base_port, client_id)


print("Start connecting to sockets")

# 3. Send Private Inputs
# The client.send_private_inputs() function handles the entire
# secret sharing and communication protocol for you.
print(f"Sending private inputs: {bonus}")
client.send_private_inputs(bonus)
print("✅ Inputs sent.")

# 4. Receive the Result
# The MPC program will reveal one value (the result of the comparison).
print("⏳ Waiting to receive result...")
result = client.receive_outputs(nr_of_outputs)
print(f"✅ Result received: {result}")
