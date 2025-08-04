import socket
import struct
import random
import os
import sys

sys.path.insert(0, os.path.dirname(sys.argv[0]) + '/..')

from Client import MASCotClient
from math import ceil

# --- Configuration ---
# The values you want to compare
VALUE_1 = 5
VALUE_2 = 10

# The host and port of Party 0
# 'party0' is the service name in docker-compose
HOST = "party0"
PORT = 10000

# Number of computation parties
N_PARTIES = 2

# The bit length for the ring, must match compile.bash
BIT_LENGTH = 64
K = BIT_LENGTH
S = 64 # Security parameter, not as critical for the client

# --- Client Logic ---

def secret_share_int(value, n_parties, mod):
    """
    Creates arithmetic shares for a single integer value.
    x = (share_0 + share_1 + ... + share_n-1) % mod
    """
    shares = [random.randrange(mod) for _ in range(n_parties - 1)]
    last_share = (value - sum(shares)) % mod
    shares.append(last_share)
    return shares

print(f"Connecting to MP-SPDZ at {HOST}:{PORT}")
print(f"Preparing to send inputs: {VALUE_1} and {VALUE_2}")
print("----------------------------------------")

# 1. Create shares for the inputs
# The MPC program expects 2 sints
mod = 2**BIT_LENGTH
shares_v1 = secret_share_int(VALUE_1, N_PARTIES, mod)
shares_v2 = secret_share_int(VALUE_2, N_PARTIES, mod)

print(f"Shares for {VALUE_1}: {shares_v1}")
print(f"Shares for {VALUE_2}: {shares_v2}")

# 2. Connect to Party 0 and send the shares
# The client sends all shares to party 0.
# Party 0 will then distribute the necessary shares to the other parties.
client_socket = MASCotClient(HOST, PORT, 0) # Client ID 0
client_socket.send_private_inputs(shares_v1)
client_socket.send_private_inputs(shares_v2)

print("\nInputs sent successfully. Waiting for result...")

# 3. Receive the result shares
# The MPC program returns 1 sint
result_shares = client_socket.receive_outputs(1)
final_result = sum(result_shares) % mod

# The result of a comparison (a < b) is 1 if true, 0 if false.
# For negative numbers, we need to handle two's complement representation.
if final_result > mod / 2:
    final_result -= mod

print("----------------------------------------")
print(f"Received result shares: {result_shares}")
print(f"Reconstructed result: {final_result}")

if final_result == 1:
    print(f"Conclusion: {VALUE_1} < {VALUE_2} is TRUE ✅")
else:
    print(f"Conclusion: {VALUE_1} < {VALUE_2} is FALSE ❌")

client_socket.close()
