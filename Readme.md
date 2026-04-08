# Chess-Server (+ Bevy Client)

My personal 'everyone needs to write a chess program in his life' attempt.

A chess program as a client-server architecture. The server is responsible for all the chess logic while clients can send commands to create, join and play chess games.

Could evolve in either a small-sized online chess or a chess analysis software if I will ever care enough.

### Short/Mid-Term TODO:

- [x] Server: Basic chess logic (looks good, but not battle tested yet)

- [x] Server: Accepting connections, handling requests for creating, joining games and making moves

- [x] Server: Managing multiple games at once

- [x] Server: Setting nicknames

- [ ] Server: 50-Moves-Rules

- [ ] Server: 3-Moves-Repetition

- [ ] Server: Insufficient Material Draw

- [ ] Server: Offer Draw

- [ ] Server: Resigning

- [x] Server: Log move history

- [ ] Server: Time control management

- [x] Client: Basic chess playing (moving pieces with mouse, promoting)

- [x] Client: Receiving messages for game states (moves made / game over)

- [x] Client: Create, list and join games

- [ ] Client: Resigning

- [ ] Client: Better connection settings (rudimentary config file exists)

- [ ] Client: Material overview

- [ ] Client: Show move history

- [ ] Client: setup game time controls

- [ ] Protocol: Algebraic Notation


### Long-Term TODO:

- [ ] Persistent Accounts

- [ ] Puzzles

- [ ] Analysis

- [ ] UCI Bridge (Stockfish integration)



### Quirks:

- [ ] Client: Pieces can be dragged outside the board

- [ ] Client: Scrollbar in games list doesn't fit very nicely


