# Chess-Server (+ Bevy Client)

My personal 'everyone needs to write a chess program in his life' attempt.

A chess program as a client-server architecture. The server is responsible for all the chess logic while clients can send commands to create, join and play chess games.

Could evolve in either a small-sized online chess or a chess analysis software if I will ever care enough.

Short-Term TODO:

[x] Server: Basic chess logic (looks good, but not battle tested yet)

[x] Server: Accepting connections, handling requests for creating, joining games and making moves

[x] Server: Managing multiple games at once

[ ] Server: 50-Moves-Rules

[ ] Server: 3-Moves-Repetition

[ ] Server: Insufficient Material Draw

[ ] Server: Offer Draw

[ ] Server: Removing finished games from game list

[ ] Server: Resigning

[ ] Server: Log move history

[ ] Server: Time control management

[x] Client: Basic chess playing (moving pieces with mouse, promoting)

[x] Client: Receiving Game Over at Checkmate

[ ] Client: Receiving Game Over at Stalemate

[ ] Client: Resigning

[ ] Client: Connection settings (only localhost/autoconnect at the moment)

[ ] Client: Material overview

[ ] Client: Show move history

[ ] Client: setup game time controls


Long-Term I'd like to add persistent accounts, the ability to save games for later review and an UCI interface to integrate stockfish.

Quirks:

[ ] Client: Pieces can be dragged outside the board
