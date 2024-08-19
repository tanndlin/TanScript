#include <stdio.h>

#include "opcodes.h"

typedef struct {
    enum Opcode opcode;
    int* operands;
    int numOperands;
} Instruction;

void printInstruction(Instruction instr, FILE* f) {
    fprintf(f, "%s", opcodeToString(instr.opcode));
    if (instr.operands != NULL) {
        fprintf(f, " -> ");
        for (int i = 0; i < instr.numOperands; i++) {
            fprintf(f, "%d ", instr.operands[i]);
        }
    }

    fprintf(f, "\n");
    fflush(f);
}
