#!/bin/env python3
from socket import *

s = socket(AF_PACKET, SOCK_RAW)
s.bind(("tap0", 0))
s.send(bytes.fromhex("deadbeefcafebed7b4e662140800450000287325400040114e49c0a8fc03c0a8fc01b39d270f001407cd7476616c76327c726563760a"))
