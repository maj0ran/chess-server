## Data Flow Visualization

This document describes the architectural data flow of this humble chess server.

```text
                               +-------------------------------------------------------------+
      +----------------------->|                         TCP Server                          |
      |       (1) connects     |  (Listens for new connections, spawns client sessions)      |
      |                        +---------------------------+---------------------------------+
      |                                                    |                                                                   
      |                                                    | (2) spawns          +-----------------------------------------+  
      |                                                    v                     |                 Game Manager            |  
+------------------+   (3)    +-------------------------------------+            |  +------------------+                   |
|  Remote Clients  | <------> |            Client Session           |<-----------|->| Client Endpoint  |                   |
|                  |   TCP    |                                     |   Channels |  +------------------+                   |
|  (Outside World) |          |         (one task per client)       |            |  | Client Endpoint  |  (Central Task,   |
|                  |          +-------------------------------------+            |  +------------------+   routes msgs,    |  
+------------------+          |            Client Session           |            |  |      ...         |   owns games)     |
                              +-------------------------------------+            |  +------------------+                   |
                              |                 ...                 |            |                                         |  
                              +-------------------------------------+            |                                         |  
                                                                                 +--------------------+--------------------+
                                                                                                      |
                                                                                                      | owns
                                                                                                      v
                                                                                             ___________________  
                                                                                            /                  /|  
                                                                                           +------------------+ |
                                                                                           |   Chess Games    | |
                                                                                           |                  | |
                                                                                           |  (State & Logic) | /
                                                                                           +------------------+
```

### Flow Description

1. **TCP Server**: Listens on a port for incoming connections.
    * **Remote Clients** first initiate a TCP connection to the **TCP Server** (`connects`).
    * The **TCP Server** `accepts` the connection and spawns a **Client Session** to handle it.
2. **Client Session**: Each session runs in its own asynchronous task. There are multiple sessions (one per connected
   client). It acts as a bridge:
    * Translates byte streams from the **Remote Client** into `ClientMessage` enums using the protocol.
    * Forwards these messages to the **Game Manager** via an unbounded MPSC channel.
    * Receives `ServerMessage` enums from the **Game Manager** and translates them back to bytes for the **Remote Client
      **.
3. **Game Manager**: The central authority running in its own task.
    * Maintains a map of `ClientId` to **Client Endpoints** (which hold the transmission half of the channel back to the
      session).
    * Owns and manages multiple **Chess Games**.
    * Processes incoming commands (Register, NewGame, JoinGame, Move) and updates the respective **Chess Game** state.
    * Broadcasts or sends targeted events (GameCreated, PlayerJoined, MoveMade) back to clients via their **Client
      Endpoints**.
4. **Chess Game**: Contains the game logic, board state (via `Chess` struct), and player information (`GameDetails`).
5. **Client Endpoint**: A handle held by the **Game Manager** to communicate back to a specific **Client Session**.
   There are multiple of these (one per session).
