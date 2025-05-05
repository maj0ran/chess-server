/*
* Frame Format
* ------------
* - first byte is length of the message
* - second byte is either:
* --- 0xA 0x32 <param> to create a new game with params
*     params are seperated by 0x32 (space)
* --- [a-h] followed by another 3-4 bytes to indicate a chess move like d2d4
*/
