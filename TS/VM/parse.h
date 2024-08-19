#include <stdbool.h>

#include "instructions.h"

#define MAX_INSTRUCTIONS 1024

int numInstructions;

int opcodeToNumOperands(enum Opcode opcode) {
    switch (opcode) {
        case ADDI:
        case SUBI:
        case MULI:
        case DIVI:
        case LESS:
        case LEQ:
        case EQ:
        case GEQ:
        case GREATER:
        case POP:
        case UNFRAME:
        case PRINTC:
        case PRINTINT:
        case STORESP:
        case STORESTACK:
        case PUSHSTACK:
        case POPSTACK:
        case RETURN:
            return 0;
        case FRAME:
        case PUSH:
        case LOAD:
        case STORE:
        case ALLOC:
        case GOTO:
        case JUMP:
        case JMPT:
        case JMPF:
            return 1;
    }

    printf("Parse Error: Unknown amount of operands for opcode: {%d (%s)}\n", opcode, opcodeToString(opcode));
    exit(1);
}

Instruction parseInstruction(FILE* file, enum Opcode opcode) {
    Instruction ret;
    ret.opcode = opcode;

    int numOperands = opcodeToNumOperands(opcode);
    ret.numOperands = numOperands;
    if (numOperands == 0) {
        ret.operands = NULL;
        return ret;
    }

    ret.operands = malloc(sizeof(int) * numOperands);
    for (int i = 0; i < numOperands; i++) {
        if (fscanf(file, "%d", &ret.operands[i]) == EOF) {
            printf("Error: Unexpected EOF\n");
            exit(1);
        }
    }
    return ret;
}

Instruction* parse(char* fileName) {
    FILE* file = fopen(fileName, "r");
    if (file == NULL) {
        printf("Error: File not found\n");
        exit(1);
    }

    fscanf(file, "%d", &numInstructions);
    if (numInstructions <= 0) {
        printf("Error: Invalid number of instructions: %d\n", numInstructions);
        exit(1);
    }

    if (numInstructions >= MAX_INSTRUCTIONS) {
        printf("Error: Too many instructions: %d\n", numInstructions);
        exit(1);
    }

    Instruction* instructions = malloc(sizeof(Instruction) * numInstructions);
    for (int i = 0; i < numInstructions; i++) {
        int opcode;
        if (fscanf(file, "%d", &opcode) == EOF) {
            break;
        }

        instructions[i] = parseInstruction(file, opcode);
    }

    fclose(file);
    return instructions;
}
