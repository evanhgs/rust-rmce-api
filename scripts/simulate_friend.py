#!/usr/bin/env python3
"""
Simule un ami connecté qui se déplace autour de Valence.
Usage: 
python3 -m venv .venv                                                                                                                                    
source .venv/bin/activate                                                                                                                                  
pip install websockets requests
python3 scripts/simulate_friend.py
"""

import asyncio
import json
import math
import time
import requests
import websockets

# ─── Config ────────────────────────────────────────────────────────────────────
API_URL = "http://localhost:5000"
GEO_URL = "ws://localhost:8080"

EMAIL    = "ami@test.com"
PASSWORD = "Password1!"

# iut 
LAT_CENTER = 44.915686
LNG_CENTER = 4.916956
# Rayon du cercle simulé (en degrés, ~500m)
RADIUS = 0.005
INTERVAL = 2.0  # secondes entre chaque position
# ───────────────────────────────────────────────────────────────────────────────


def login() -> str:
    print(f"[+] Connexion en tant que {EMAIL}...")
    resp = requests.post(
        f"{API_URL}/auth/login",
        json={"email": EMAIL, "password": PASSWORD},
        timeout=5,
    )
    resp.raise_for_status()
    token = resp.json()["token"]
    print(f"[+] Token obtenu : {token[:30]}...")
    return token


async def stream_location(token: str):
    uri = f"{GEO_URL}/ws?token={token}"
    print(f"[+] Connexion WebSocket : {uri}")

    async with websockets.connect(uri) as ws:
        print("[+] Connecté ! Envoi de la position toutes les 2s (Ctrl+C pour arrêter)\n")
        angle = 0.0
        try:
            while True:
                # Cercle autour de Valence
                lat = LAT_CENTER + RADIUS * math.sin(math.radians(angle))
                lng = LNG_CENTER + RADIUS * math.cos(math.radians(angle))

                payload = json.dumps({"lat": lat, "lng": lng})
                await ws.send(payload)

                print(f"  → lat={lat:.6f}  lng={lng:.6f}  (angle={angle:.0f}°)")

                # Écouter les messages entrants (positions d'autres amis)
                try:
                    msg = await asyncio.wait_for(ws.recv(), timeout=0.1)
                    print(f"  ← reçu: {msg}")
                except asyncio.TimeoutError:
                    pass

                angle = (angle + 10) % 360
                await asyncio.sleep(INTERVAL)

        except KeyboardInterrupt:
            print("\n[+] Arrêt.")


async def main():
    token = login()
    await stream_location(token)


if __name__ == "__main__":
    asyncio.run(main())
