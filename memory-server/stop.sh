#!/bin/bash
PID_FILE="$HOME/.memory-system/server.pid"
if [ -f "$PID_FILE" ]; then
    kill "$(cat "$PID_FILE")" 2>/dev/null && echo "Memory server stopped" || echo "Not running"
    rm -f "$PID_FILE"
else
    echo "No PID file found"
fi
