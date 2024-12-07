(START)
// Set cursor to the beginning of the screen memory region
@SCREEN
D=A
@cur
M=D
(LOOP)
// Check keyboard input
@KBD
D=M
// if no input go to clear
@CLR
D;JEQ
// else go to filling
@fill
M=-1
// jump to filling loop
@FILL
0;JMP
(CLR)
@fill
M=0
(FILL)
@fill
D=M
@cur
A=M
M=D
@cur
M=M+1
// if reached end of the screen mem region, restart 
@SCREEN
D=A
@8192
D=D+A
@cur
D=D-M
@START
D;JEQ
// else continue looping
@LOOP
0;JMP
