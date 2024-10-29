# Simple Websocket

A websocket sample made in Rust using Actix Framework Websocket.

---

Currently only for local testing. To run it in local just do `cargo run`. And then connect with something like `wscat` to websocket endpoint:

``` wscat -c ws://127.0.0.1:8080/ws ```

Available command:
- `/subscribe {room_name}`
- `/unsubscribe {room_name}`
- `/list`
- `/send {room_name} {message}`

---

With session ID you get from websocket connection, it also can take request via REST API endpoints:

- `/send`
``` curl -X POST -H "Content-Type: application/json" -d '{"message": "Hello", "room_name": "testroom", "session_id": "8b7b4572-1753-4625-a2bf-36fc37c3b4db"}' http://127.0.0.1:8080/send ```

- `/subscribe`
``` curl -X POST -H "Content-Type: application/json" -d '{"message": "Hello", "room_name": "testroom", "session_id": "8b7b4572-1753-4625-a2bf-36fc37c3b4db"}' http://127.0.0.1:8080/send ```

- `/unsubscribe`
``` curl -X POST -H "Content-Type: application/json" -d '{"message": "", "room_name": "testroom", "session_id": "8b7b4572-1753-4625-a2bf-36fc37c3b4db"}' http://127.0.0.1:8080/unsubscribe ```

- `/list`
``` curl -X POST -H "Content-Type: application/json" -d '{"message": "", "room_name": "", "session_id": "8b7b4572-1753-4625-a2bf-36fc37c3b4db"}' http://127.0.0.1:8080/list ```

---

To Do:
- Handle request if missing parameter
- Reorganize scripts to files