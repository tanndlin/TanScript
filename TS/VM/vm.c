#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#include "parse.h"

Instruction* parse(char* fileName, int* size);

Instruction* instructions;
int pc = 0;

int* stack;
int sp = 0;

int main(int argc, char* argv[]) {
    if (argc != 2) {
        printf("Usage: %s <file>\n", argv[0]);
        exit(1);
    }

    int numInstructions;
    instructions = parse(argv[1], &numInstructions);
    // print all instructions
    for (int i = 0; i < numInstructions; i++)
        printInstruction(&instructions[i]);

    stack = malloc(sizeof(int) * 100);

    return 0;
}

void runLine() {
    Instruction instr = instructions[pc];
    switch (instr.opcode) {
        case ADDI:
            stack[sp - 2] = stack[sp - 2] + stack[sp - 1];
            sp--;
            break;
        case SUBI:
            stack[sp - 2] = stack[sp - 2] - stack[sp - 1];
            sp--;
            break;
        case MULI:
            stack[sp - 2] = stack[sp - 2] * stack[sp - 1];
            sp--;
            break;
        case DIVI:
            stack[sp - 2] = stack[sp - 2] / stack[sp - 1];
            sp--;
            break;
        case PUSH:
            stack[sp] = instr.operands[0];
            sp++;
            break;
        case POP:
            sp--;
            break;
    }

    pc++;
}
