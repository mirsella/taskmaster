#!/usr/bin/python3

import os
import random
import signal
from time import sleep

sleep(3)

decider = random.randint(0, 2)

if decider == 0:
    exit(random.randint(0, 255))
elif decider == 1:
    sleep(10)
    os.kill(os.getpid(), signal.SIGSEGV)
sleep(20)