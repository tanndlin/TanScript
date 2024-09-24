# TanScript

A custom programming language created with Test Driven Development.

## Tokens

| Name       | Args                 | Formula                                                                    | Description                                                                   |
| ---------- | -------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| ADDI       | -                    | `stack[sp-2] = stack[sp-2] + stack[sp-1];`<br>`sp--;`                      | Adds the previous 2 items on the stack and stores in place                    |
| SUBI       | -                    | `stack[sp-2] = stack[sp-2] - stack[sp-1];`<br>`sp--;`                      | Subtracts the previous 2 items on the stack and stores in place               |
| MULI       | -                    | `stack[sp-2] = stack[sp-2] \* stack[sp-1];`<br>`sp--;`                     | Multiplies the previous 2 items on the stack and stores in place              |
| DIVI       | -                    | `stack[sp-2] = stack[sp-2] +/ stack[sp-1];`<br>`sp--;`                     | Divides the previous 2 items on the stack and stores in place                 |
| LESS       | -                    | `stack[sp-2] = stack[sp-2] < stack[sp-1];`<br>`sp--;`                      | Compares the previous 2 items on the stack and stores in place                |
| LEQ        | -                    | `stack[sp-2] = stack[sp-2] <= stack[sp-1];`<br>`sp--;`                     | Compares the previous 2 items on the stack and stores in place                |
| EQ         | -                    | `stack[sp-2] = stack[sp-2] == stack[sp-1];`<br>`sp--;`                     | Compares the previous 2 items on the stack and stores in place                |
| GEQ        | -                    | `stack[sp-2] = stack[sp-2] >= stack[sp-1];`<br>`sp--;`                     | Compares the previous 2 items on the stack and stores in place                |
| GREATER    | -                    | `stack[sp-2] = stack[sp-2] > stack[sp-1];`<br>`sp--;`                      | Compares the previous 2 items on the stack and stores in place                |
| PUSH       | n                    | `stack[sp++] = n;`                                                         | Pushes a value onto the stack                                                 |
| POP        | -                    | `sp--;`                                                                    | Pops a value off the stack                                                    |
| LOAD       | offset               | `stack[sp++] = stack[bp + offset];`                                        | Loads a value from memory into the stack                                      |
| STORE      | offset               | `stack[bp + offset] = stack[--sp];`                                        | Stores a value from the stack into memory                                     |
| ALLOC      | n                    | `sp += n;`                                                                 | Allocates n spaces on the stack                                               |
| FRAME      | offset?              | `stack[sp++] = pc + offset;`                                               | Creates a new frame on the stack, storing the current instruction number      |
| UNFRAME    | -                    | `pc = stack[sp -1];`<br>`stack[sp-1] = returnValue;`<br>`reutrnValue = 0;` | Removes the current frame from the stack, jumping to the stored pc value      |
| STORESP    | -                    | `stack[sp] = sp;`<br>`sp++;`                                               | Stores the current stack pointer in the stack                                 |
| STORESTACK | -                    | `stack[stack[sp-1]+bp] = stack[sp-2];`<br>`sp -= 2;`                       | Stores the value at `sp-1` in the stack at the location `sp-2`+bp             |
| GOTO       | absolute_line_number | `pc = absolute_line_number;`                                               | Jumps to the absolute line number                                             |
| JUMP       | relative_line_number | `pc += relative_line_number;`                                              | Offsets the pc by the relative line number                                    |
| JMPT       | relative_line_number | `if (stack[--sp]) pc += relative_line_number;`                             | Jumps to the relative line number if the top of the stack is true             |
| JMPF       | relative_line_number | `if (!stack[--sp]) pc += relative_line_number;`                            | Jumps to the relative line number if the top of the stack is false            |
| PRINTC     | -                    | `print(stack[--sp]);`                                                      | Prints the top of the stack as a character                                    |
| PRINTINT   | -                    | `print(stack[--sp]);`                                                      | Prints the top of the stack as an integer                                     |
| PUSHSTACK  | -                    | `stack[sp++] = bp;`<br>`bp = sp;`                                          | Reserves memory for the current frame by pushing the BP, and setting BP to SP |
| POPSTACK   | -                    | `sp = bp;`<br>`bp = stack[--sp];`                                          | Restores the previous frame by setting SP to BP, and popping BP               |
| RETURN     | -                    | `returnValue = stack[--sp];`                                               | Sets `returnValue` to the top of the stack                                    |
