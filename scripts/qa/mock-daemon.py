#!/usr/bin/env python3
"""Protocol-compatible Hyprlume daemon stub for shell QA.

This is a test fixture, not a daemon implementation. It speaks the same
newline-delimited JSON protocol as src/server/ so the QML shell can be
exercised without a Rust build. Every state it returns is synthetic.

Usage:
    python3 scripts/qa/mock-daemon.py /tmp/hyprlume-qa.sock
    HYPRLUME_SOCKET=/tmp/hyprlume-qa.sock qs -p shell/shell.qml
"""

import asyncio
import json
import os
import signal
import sys

STATE = {
    "version": "0.1.0-qa",
    "phase": "ready",
    "message": "Your desktop, within reach.",
    "transcript": "",
    "actions": [],
    "receipts": [],
    "confirmation": None,
    "panel_open": True,
    "voice": "off",
    "workspace": 1,
    "active_app": "kitty",
}

CONNECTIVITY = {
    "panel_open": False,
    "busy": False,
    "message": "Synthetic wireless services are unavailable in this IPC fixture.",
    "error": "",
    "wifi": {"available": False, "enabled": False, "hardware_enabled": False,
             "interfaces": [], "networks": [], "error": "No fixture Wi-Fi adapter"},
    "bluetooth": {"available": False, "adapters": [], "devices": [],
                  "error": "No fixture Bluetooth adapter"},
    "prompt": None,
}

NOTIFICATIONS = {"available": False, "error": "No notification server in this IPC fixture", "panel_open": False, "dnd": False, "unread": 0, "items": []}
TOPICS = {"state": STATE, "connectivity": CONNECTIVITY, "notifications": NOTIFICATIONS}
CLIENTS = {}
PLAN_TASK = None


def wire(state, topic="state"):
    return (json.dumps({"event": topic, "state": state}) + "\n").encode()


async def broadcast(topic="state"):
    payload = wire(TOPICS[topic], topic)
    for writer, topics in list(CLIENTS.items()):
        if topic not in topics:
            continue
        try:
            writer.write(payload)
            await writer.drain()
        except (ConnectionError, RuntimeError):
            CLIENTS.pop(writer, None)


def reset_plan():
    global PLAN_TASK
    if PLAN_TASK and not PLAN_TASK.done():
        PLAN_TASK.cancel()
    PLAN_TASK = None


async def plan_later(dry_run):
    await asyncio.sleep(0.7)
    if dry_run:
        STATE.update(
            phase="ready",
            message="Review the plan before it runs.",
            actions=[
                "Focus workspace 2",
                "Open kitty",
                "Notify: workspace ready",
            ],
            receipts=[],
            confirmation=42,
        )
    else:
        STATE.update(
            phase="ready",
            message="Done. Nothing else is queued.",
            actions=[],
            receipts=["Focused workspace 2", "Opened kitty"],
            confirmation=None,
        )
    await broadcast()


async def handle(reader, writer):
    global PLAN_TASK
    CLIENTS[writer] = set()
    try:
        while True:
            line = await reader.readline()
            if not line:
                break
            try:
                request = json.loads(line)
            except json.JSONDecodeError:
                writer.write(b'{"id":null,"error":"invalid request"}\n')
                await writer.drain()
                continue

            operation = request.get("op")
            request_id = request.get("id")

            if operation in ("subscribe", "connectivity_subscribe", "connectivity_status", "notifications_subscribe", "notifications_status"):
                topic = "state" if operation == "subscribe" else operation.rsplit("_", 1)[0]
                if operation.endswith("subscribe"):
                    CLIENTS[writer].add(topic)
                writer.write((json.dumps({"id": request_id, "result": TOPICS[topic]}) + "\n").encode())
                await writer.drain()
                continue

            if operation == "notifications":
                action = request.get("request", {})
                if action.get("action") == "toggle":
                    NOTIFICATIONS["panel_open"] = not NOTIFICATIONS["panel_open"]
                    if NOTIFICATIONS["panel_open"]:
                        STATE["panel_open"] = CONNECTIVITY["panel_open"] = False
                elif action.get("action") == "close":
                    NOTIFICATIONS["panel_open"] = False
                elif action.get("action") == "dnd":
                    NOTIFICATIONS["dnd"] = bool(action.get("enabled"))
                writer.write((json.dumps({"id": request_id, "result": NOTIFICATIONS}) + "\n").encode())
                await writer.drain()
                for topic in TOPICS:
                    await broadcast(topic)
                continue

            if operation == "connectivity":
                action = request.get("request", {}).get("action")
                if action == "toggle":
                    CONNECTIVITY["panel_open"] = not CONNECTIVITY["panel_open"]
                    if CONNECTIVITY["panel_open"]:
                        STATE["panel_open"] = False
                        NOTIFICATIONS["panel_open"] = False
                        await broadcast("notifications")
                elif action == "close":
                    CONNECTIVITY["panel_open"] = False
                elif action != "refresh":
                    writer.write((json.dumps({"id": request_id, "error": "No wireless hardware in this fixture"}) + "\n").encode())
                    await writer.drain()
                    continue
                writer.write((json.dumps({"id": request_id, "result": CONNECTIVITY}) + "\n").encode())
                await writer.drain()
                await broadcast("connectivity")
                await broadcast()
                continue

            if operation == "toggle":
                STATE["panel_open"] = not STATE["panel_open"]
                if STATE["panel_open"]:
                    NOTIFICATIONS["panel_open"] = False
                    await broadcast("notifications")
                    CONNECTIVITY["panel_open"] = False
                    await broadcast("connectivity")
            elif operation == "command":
                reset_plan()
                dry_run = request.get("dry_run", True)
                STATE.update(
                    phase="planning",
                    message="Planning a safe route...",
                    transcript=request.get("text", ""),
                    actions=[],
                    receipts=[],
                    confirmation=None,
                )
                PLAN_TASK = asyncio.create_task(plan_later(dry_run))
            elif operation == "confirm":
                reset_plan()
                STATE.update(
                    phase="ready",
                    message="Done. Nothing else is queued.",
                    actions=[],
                    receipts=["Focused workspace 2", "Opened kitty"],
                    confirmation=None,
                )
            elif operation == "cancel":
                reset_plan()
                STATE.update(
                    phase="ready",
                    message="Preview dismissed",
                    actions=[],
                    receipts=[],
                    confirmation=None,
                )
            elif operation == "voice":
                mode = request.get("mode", "off")
                STATE["voice"] = mode
                STATE["panel_open"] = True
                if mode == "wake":
                    STATE.update(phase="listening", message="Listening for the wake word.")
                elif mode == "dictate":
                    STATE.update(phase="listening", message="Listening. Speak your command.")
                else:
                    STATE.update(phase="ready", message="Microphone off.")
            elif operation != "status":
                writer.write(
                    (json.dumps({"id": request_id, "error": "unknown operation"}) + "\n").encode()
                )
                await writer.drain()
                continue

            writer.write((json.dumps({"id": request_id, "result": STATE}) + "\n").encode())
            await writer.drain()
            await broadcast()
    except (ConnectionError, asyncio.CancelledError):
        pass
    finally:
        CLIENTS.pop(writer, None)
        writer.close()


async def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/hyprlume-qa.sock"
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass

    server = await asyncio.start_unix_server(handle, path=path)
    os.chmod(path, 0o600)
    print(f"mock daemon listening on {path}", flush=True)

    stop = asyncio.Event()
    loop = asyncio.get_running_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, stop.set)

    async with server:
        await stop.wait()


if __name__ == "__main__":
    asyncio.run(main())
