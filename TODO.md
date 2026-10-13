Function body actual return type needs to match the declared return type

Memory-destination ops (add/sub/cmp/... on a stack slot) need their operand size from the destination type, not the immediate, or narrower locals get clobbered
