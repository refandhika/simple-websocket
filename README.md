# Simple Websocket

A websocket sample made in Rust using Actix Framework Websocket.

---

Currently only for local testing. To run it in local just do `cargo run`. And then connect with something like `wscat` to websocket endpoint:

``` wscat -c ws://127.0.0.1:8080/ws ```

---

To Do:
- Add a send message API under `/send`