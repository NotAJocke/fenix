#!/usr/bin/env python3
"""Random AI for Fenix. Pick legal moves at random."""

import json
import random
import sys
import time
import urllib.request

API = "http://127.0.0.1:3000"


def get(path):
    for _ in range(30):
        try:
            return json.loads(urllib.request.urlopen(f"{API}{path}", timeout=5).read())
        except Exception:
            time.sleep(0.5)
    raise SystemExit("Server not reachable")


def post(path, data):
    body = json.dumps(data).encode()
    req = urllib.request.Request(f"{API}{path}", body, {"Content-Type": "application/json"})
    for _ in range(30):
        try:
            return json.loads(urllib.request.urlopen(req, timeout=5).read())
        except Exception:
            time.sleep(0.5)
    raise SystemExit("Server not reachable")


def main():
    color = sys.argv[1] if len(sys.argv) > 1 else ""
    color = color.capitalize()

    while True:
        r = get("/state")
        phase = r["phase"]

        if phase == "GameOver":
            print(f"Game over")
            break

        if color and r["turn"] != color:
            time.sleep(0.1)
            continue

        moves = r["legal_moves"]
        if not moves:
            print("No legal moves")
            break

        move = random.choice(moves)
        print(f"{r['turn']}: {move['from']} -> {move['to']}")

        post("/move", {"from": move["from"], "to": move["to"]})
        time.sleep(0.2)


if __name__ == "__main__":
    main()
