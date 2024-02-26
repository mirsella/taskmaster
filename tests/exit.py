#!/usr/bin/python3

import random
from time import sleep

should_exit = random.randint(0, 1)

if should_exit:
    exit(random.randint(0, 255))

sleep(20)