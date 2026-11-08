#!/bin/sh
# Start an in-memory SurrealDB for the corpus on 127.0.0.1:8765 (podman), then load it.
podman rm -f rg-surreal >/dev/null 2>&1
podman run -d --name rg-surreal -p 127.0.0.1:8765:8000 docker.io/surrealdb/surrealdb:v4.0.0 start --user root --pass root memory >/dev/null
sleep 2
exec python3 "$(dirname "$0")/rg.py" load
