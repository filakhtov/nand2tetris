# Nand 2 Tetris
This is my journey through the Elements of Computing Systems, second edition -
building a modern computer from first principles by Noam Nisan and Shimon
Schocken.

The `hdl` directory contains elementary logic gate implementations, as well as
advanced chips, such as ALU, registers and RAM. They are defined using the
simplified HDL (Hardware Description Language).

Elementary logic gates are done as the first project in the book and are the
foundational components that are used to build all other advanced chips.

Second project guides through implementation of more advanced components, like
half-adder, full-adder, 16-bit adder and 16-bit incrementer. Once all of these
components are implemented they are used to implement a 16-bit ALU.

Third project involves development of memory registers and various RAM modules,
starting from a single bit register, then 16-bit register, then a set of RAM
modules with 8, 64, 512, 4k and 16k 16-bit registers. A Program Counter (PC) is
also implemented in this project.

The `asm` directory contains various assembly programs from different stages in
journey.

The `Mult.asm` is developed as part of the fourth project and is designed to
multiply two numbers stored in `RAM[0]` and `RAM[1]` and save the result of the
computation in the `RAM[2]` register. It is guaranteed that input numbers will
be greater or equal to zero and the output number will be less than 32768.
