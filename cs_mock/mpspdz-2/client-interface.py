#!/usr/bin/python3
# 
# import sys, random
# 
# #sys.path.insert(0, 'ExternalIO')
# sys.path.append('.')
# 
# print("Running Client")
# 
# from client import *
# from domains import *
# print("After import")
# 
# base_port = 10000
# print("Creating client")
# 
# client_id = int(sys.argv[1])
# bonus = list(map(lambda x: int(x), sys.argv[2:]))
# 
# client = Client(['party0', 'party1'], base_port, client_id)
# 
# 
# print("Start connecting to sockets")
# 
# # 3. Send Private Inputs
# # The client.send_private_inputs() function handles the entire
# # secret sharing and communication protocol for you.
# print(f"Sending private inputs: {bonus}")
# client.send_private_inputs(bonus)
# print("✅ Inputs sent.")
# 
# # 4. Receive the Result
# # The MPC program will reveal one value (the result of the comparison).
# print("⏳ Waiting to receive result...")
# result = client.receive(1)
# print("✅ Result received.")


import sys, random

sys.path.insert(0, 'ExternalIO')

from client import *

party = int(sys.argv[1])

client = Client(['party0', 'party1'], 15000 + party, 0)

n = 1000

if party < 2:
    client.send_public_inputs(random.gauss(0, 1) * 2 ** 16 for i in range(n))

x = [client.receive_plain_values(client.sockets[i]) for i in range(2)]
print(x)
#client.send_public_inputs(a + b for a, b in zip(*x))
