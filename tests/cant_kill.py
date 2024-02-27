import signal
import sys
from time import sleep

def signal_handler(sig, frame):
    print('I don\'t care')

signal.signal(signal.SIGTERM, signal_handler)

while True:
    sleep(10000000)