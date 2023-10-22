All messages are terminated with '\0'.

The protocol uses where possible 1 byte sequences for commands. Instead of writing the value of byte, the command-identifiers are chosen to represent a printable character in Standard ASCII.
Hence when the documentation talks about the command 'J', the implementation should send the byte 0x4A or the decimal integer 74.

Client is in 1 of 2 modes: 0 or P.

0 is the default mode the client is after connecting. It represents not being in an active game of chess.
P is the play mode. In this mode, move commands from the client are interpreted by the server.

**in 0, following messages of the client are parsed:**

N {0,1,2} // Create a new Game. Second parameter is 0: play as black, : play as white, 2: play as random
J id // Join game by its unique id.


**in G, following messages of the client are parsed:**

Move-Command in the form {square}{square}[promotion] as in d2d4, f8c6, f7f8Q.

'D' // offer draw. If the opponent has currently offered  a draw that is not declined, this accepts the draw.
'E' // decline draw.

**Response**

If in Game Mode, a move will be answered by a respond. The first byte of the respond is the length of the message.
Each 2 following bytes indicate a field as a linear memory index (a8 = 0, h1 = 63) and a value that represents a piece. This tuple indicates a field that has been updated with a piece.

if Length is 1, then the move was not executed and no change happened. the first byte then indicates an error type (e.g. illegal move).

value for pieces are:

BLACK = 0
WHITE = 1

KING = 1 << 1
QUEEN = 1 << 2
ROOK = 1 << 3
BISHOP = 1 << 4
KNIGHT = 1 << 5
PAWN = 1 << 6
