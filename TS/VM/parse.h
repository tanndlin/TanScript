#include <stdbool.h>

#include "instructions.h"

Instruction* parseInstruction(FILE* file, enum Opcode opcode) {
    Instruction* ret = malloc(sizeof(Instruction));
    ret->opcode = opcode;

    if (opcode == POP) {
        ret->operands = NULL;
    } else {
        ret->numOperands = 2;
        ret->operands = malloc(sizeof(int) * ret->numOperands);
        fscanf(file, "%d %d", &ret->operands[0], &ret->operands[1]);
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
