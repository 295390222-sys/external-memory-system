#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
DB="${MEMORY_DB:-$HOME/.memory-system/memory.db}"
mkdir -p "$(dirname "$DB")"
nohup "$DIR/../venv/bin/python3" "$DIR/server.py" --db "$DB" --port 50051 \
    > "$HOME/.memory-system/server.log" 2>&1 &
echo $! > "$HOME/.memory-system/server.pid"
echo "Memory server started (PID: $(cat "$HOME/.memory-system/server.pid"))"
