# Simple Websocket

A websocket sample made in Rust using Actix Framework Websocket.

---

Currently only for local testing. To run it in local just do `cargo run`. And then connect with something like `wscat` to websocket endpoint:

``` wscat -c ws://127.0.0.1:8080/ws ```

We can also broadcast message via `/send` POST endpoint, here a sample with curl:

``` curl -X POST -H "Content-Type: application/json" -d '{"message": "Hello, I'ts A broadcast Message!"}' http://127.0.0.1:8080/send ```

---

To Do:
- Reorganize scripts to files