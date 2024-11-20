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

The `Fill.asm` is the second part of the fourth project and provides an
opportunity to handle I/O using the machine language. When any keyboard button
is pressed, every single screen pixel will be progressively lit. Once the button
is released, all pixels will progressively turn off.

The fifth and the final hardware project consists of the CPU, Memory and a whole
computer chips.

The `Memory.hdl` represents a RAM chip plus keyboard and screen memory mapping.
Given an address the memory chip will output a currently pressed keyboard
character, screen region or RAM register, depending on the address. If the load
bit is set and the address points either to a screen region or RAM region, the
Memory chip will write `in` bits into the respective region.

The `CPU.hdl` represents a final CPU assembly responsible for running the fetch-
execute cycle, fetching the instructions from ROM instruction memory, performing
the instruction decoding, reading the data from RAM, performing the compuntation
and storing the results into the RAM, then deciding the next instruction
address.
