#include <stdbool.h>

#include "instructions.h"

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
        case FRAME:
            return 0;
        case PUSH:
            return 1;
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

Instruction* parseInstruction(FILE* file, enum Opcode opcode) {
    Instruction* ret = malloc(sizeof(Instruction));
    ret->opcode = opcode;

    int numOperands = opcodeToNumOperands(opcode);
    ret->numOperands = numOperands;
    if (numOperands == 0) {
        ret->operands = NULL;
        return ret;
    }

    ret->operands = malloc(sizeof(int) * numOperands);
    for (int i = 0; i < numOperands; i++) {
        if (fscanf(file, "%d", &ret->operands[i]) == EOF) {
            printf("Error: Unexpected EOF\n");
            exit(1);
        }
    }
    return ret;
}

Instruction* parse(char* fileName, int* size) {
    FILE* file = fopen(fileName, "r");
    if (file == NULL) {
        printf("Error: File not found\n");
        exit(1);
    }

    Instruction* instructions = malloc(sizeof(Instruction) * 100);
    *size = 0;
    while (true) {
        int opcode;
        if (fscanf(file, "%d", &opcode) == EOF) {
            break;
        }

        Instruction* instr = parseInstruction(file, opcode);
        instructions[*size] = *instr;
        (*size)++;
    }
    fclose(file);
    return instructions;
}
