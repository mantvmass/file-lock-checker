import time
import sys

if len(sys.argv) != 2:
    print("Usage: python lock_test.py <filename>")
    sys.exit(1)

filename = sys.argv[1]
print(f"Locking file: {filename}")
print("Press Ctrl+C to release lock")

try:
    with open(filename, 'w+b') as f:
        while True:
            time.sleep(1)
except KeyboardInterrupt:
    print("Released lock")