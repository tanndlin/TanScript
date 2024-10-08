#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "parse.h"

#define DEBUG true
#define MAX_STACK_SIZE 1024

void validateStackSize(int n);
bool checkInvariants();
void runLine();
void freeAll();

Instruction* instructions;
int pc = 0;

int* stack;
int sp = 0;
int bp = 0;

int returnValue = 0;

int main(int argc, char* argv[]) {
    if (argc < 2) {
        printf("Usage: %s <file>\n", argv[0]);
        exit(1);
    }

    instructions = parse(argv[1]);
    // print all instructions
    if (DEBUG) {
        FILE* f = fopen("script.txt", "w");
        for (int i = 0; i < numInstructions; i++)
            printInstruction(instructions[i], f);
        fclose(f);
    }

    stack = malloc(sizeof(int) * MAX_STACK_SIZE);
    while (pc < numInstructions) {
        runLine();
    }

    freeAll();
    return 0;
}

void validateStackSize(int n) {
    if (sp - bp < n) {
        printf("Error: Not enough operands on stack\n");
        exit(1);
    }
}

bool checkInvariants() {
    if (sp < 0 || sp > MAX_STACK_SIZE) {
        printf("Error: Stack pointer out of bounds: %d\n", sp);
        return false;
    }

    if (bp < 0 || bp > MAX_STACK_SIZE) {
        printf("Error: Base pointer out of bounds: %d\n", bp);
        return false;
    }

    if (bp > sp) {
        printf("Error: Base pointer above stack pointer\n");
        return false;
    }

    if (pc < 0 || pc >= numInstructions) {
        printf("Error: Program counter out of bounds\n");
        return false;
    }

    return true;
}

void runLine() {
    if (!checkInvariants()) {
        exit(1);
    }

    Instruction instr = instructions[pc];
    if (DEBUG) {
        printf("Running: ");
        printInstruction(instr, stdout);
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
        case MODI:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] % stack[sp - 1];
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
        case NEQ:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] != stack[sp - 1];
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
        case AND:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] && stack[sp - 1];
            sp--;
            break;
        case OR:
            validateStackSize(2);
            stack[sp - 2] = stack[sp - 2] || stack[sp - 1];
            sp--;
            break;
        case NOT:
            validateStackSize(1);
            stack[sp - 1] = !stack[sp - 1];
            break;
        case PUSH:
            stack[sp] = instr.operands[0];
            sp++;
            break;
        case POP:
            sp--;
            break;
        case LOAD: {
            int address = instr.operands[0] + bp;
            stack[sp] = stack[address];
            sp++;
            break;
        }
        case STORE: {
            int address = instr.operands[0] + bp;
            stack[address] = stack[--sp];
            break;
        }
        case ALLOC:
            // for (int i = 0; i < instr.operands[0]; i++)
            // stack[sp++] = 0;
            sp += instr.operands[0];
            break;
        case FRAME:
            stack[sp] = pc + instr.operands[0];
            sp++;
            break;
        case UNFRAME:
            validateStackSize(1);
            pc = stack[sp - 1];
            stack[sp - 1] = returnValue;
            returnValue = 0;
            break;
        case STORESP:
            stack[sp] = sp;
            sp++;
            break;
        case STORESPOFFSET:
            stack[sp] = sp - bp;
            sp++;
            break;
        case STORESTACK: {
            validateStackSize(2);
            int address = stack[sp - 1] + bp;
            int value = stack[sp - 2];
            stack[address] = value;
            sp -= 2;
            break;
        }
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
        case PRINTC:
            validateStackSize(1);
            printf("%c", stack[sp - 1]);
            sp--;
            break;
        case PRINTINT:
            validateStackSize(1);
            printf("%d", stack[sp - 1]);
            sp--;
            break;
        case PUSHSTACK:
            // Store previous base pointer
            stack[sp] = bp;
            sp++;
            bp = sp;
            break;
        case POPSTACK:
            // Restore previous base pointer
            sp = bp;
            bp = stack[sp - 1];
            sp--;
            break;
        case RETURN:
            returnValue = stack[sp - 1];
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
            if (i == bp)
                printf("| ");
            printf("%d ", stack[i]);
        }
        printf("\nPC: %d\n\n", pc);
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