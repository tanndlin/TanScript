#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#include "parse.h"

#define DEBUG false
#define MAX_STACK_SIZE 2048

void validateStackSize(int n);
void runLine();
void freeAll();

Instruction* instructions;
int pc = 0;

int* stack;
int sp = 0;
int bp = 0;

int main(int argc, char* argv[]) {
    if (argc != 2) {
        printf("Usage: %s <file>\n", argv[0]);
        exit(1);
    }

    instructions = parse(argv[1]);
    // print all instructions
    if (DEBUG)
        for (int i = 0; i < numInstructions; i++)
            printInstruction(instructions[i]);

    stack = malloc(sizeof(int) * MAX_STACK_SIZE);
    while (pc < numInstructions) {
        runLine();
    }

    freeAll();
    return 0;
}

void validateStackSize(int n) {
    if (sp < n) {
        printf("Error: Not enough operands on stack\n");
        exit(1);
    }
}

void runLine() {
    Instruction instr = instructions[pc];
    if (DEBUG) {
        printf("Running: ");
        printInstruction(instr);
    }

    switch (instr.opcode) {
        case ADDI:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] + stack[sp - 1];
            sp--;
            break;
        case SUBI:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] - stack[sp - 1];
            sp--;
            break;
        case MULI:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] * stack[sp - 1];
            sp--;
            break;
        case DIVI:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] / stack[sp - 1];
            sp--;
            break;
        case LESS:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] < stack[sp - 1];
            sp--;
            break;
        case LEQ:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] <= stack[sp - 1];
            sp--;
            break;
        case EQ:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] == stack[sp - 1];
            sp--;
            break;
        case GEQ:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] >= stack[sp - 1];
            sp--;
            break;
        case GREATER:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] > stack[sp - 1];
            sp--;
            break;
        case PUSH:
            stack[sp] = instr.operands[0];
            sp++;
            break;
        case POP:
            sp--;
            break;
        case LOAD: {
            int address = instr.operands[0];
            stack[sp] = stack[address];
            sp++;
            break;
        }
        case STORE: {
            int address = instr.operands[0];
            stack[address] = stack[sp - 1];
            sp--;
            break;
        }
        case ALLOC:
            sp += instr.operands[0];
            break;
        case FRAME:
            stack[sp] = pc;
            sp++;
            break;
        case GOTO:
            pc = instr.operands[0];
            break;
        case JUMP:
            pc += instr.operands[0];
            break;
        case JMPT:
            validateStackSize(1);
            if (stack[sp - 1] != 0)
                pc += instr.operands[0];
            sp--;
            break;
        case JMPF:
            validateStackSize(1);
            if (stack[sp - 1] == 0)
                pc += instr.operands[0];
            sp--;
            break;
        case PRINT:
            validateStackSize(1);
            printf("%c", stack[sp - 1]);
            sp--;
            break;
        default:
            printf("Error: Unknown opcode: %d\n", instr.opcode);
            exit(1);
    }

    pc++;

    if (DEBUG) {
        printf("Stack: ");
        for (int i = 0; i < sp; i++) {
            printf("%d ", stack[i]);
        }
        printf("\n");
    }
}

void freeAll() {
    for (int i = 0; i < numInstructions; i++) {
        if (instructions[i].operands != NULL)
            free(instructions[i].operands);
    }

    free(instructions);
    free(stack);
}