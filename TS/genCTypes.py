with open("src/Compilation/Opcodes.ts", "r") as f:
    lines = [l for l in f.readlines() if l]

    # First line is the export const enum Opcode {, last line is }
    enums = lines[1:-1]
    enums = [l.strip().replace(",", "") for l in enums]

    print(enums)

    with open("VM/opcodes.h", "w") as f:
        f.write("enum Opcode {\n")
        for e in enums:
            f.write(f"    {e},\n")

        f.write("};\n")
