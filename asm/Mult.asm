// i = 0
@i
M=0
// res = 0
@res
M=0
(LOOP)
// if i == RAM[0] goto END
@i
D=M
@R0
D=D-M
@DONE
D;JEQ
// result += RAM[1]
@R1
D=M
@res
M=D+M
// i++
@i
M=M+1
@LOOP
0;JMP
// RAM[2] = res
(DONE)
@res
D=M
@R2
M=D
// infinite loop at the end of the program
(END)
@END
0;JMP
