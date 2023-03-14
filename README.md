# multiplex

Simple tool (reverse proxy) for harassing a TCP server by hooking up multiple clients to it.

Extremely barebones, limited to no error handling currently.

Packets sent by the server will be replicated to all clients, packets sent by clients will be sent in a single stream, in order of arrival.

## Example
These commands should be run concurrently, but in this order:
```bash
ncat -l 1337  # starts a server on :1337
multiplex 8000 127.0.0.1:1337  # starts multiplex on port 8000, connected to localhost:1337
ncat localhost 8000  # client #1
ncat localhost 8000  # client #2
```
All messages typed in the server will appear on both clients. A message typed in any client will be proxied to the server.