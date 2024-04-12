import * as fs from 'fs';
import { AST } from './AST/AST';
import { AddASTNode, NumberASTNode } from './AST/NumberAST';
import { CompilerError } from './errors';
import { Token } from './types';

export default class Compiler {
    constructor(private readonly ast: AST) {}

    compile() {
        const generated = this.generateAssembly();

        const assembly = `section .text
    global _main
    extern _printf
        
 _main:
    ${generated}`;
        fs.writeFileSync('out.asm', assembly);
    }

    public generateAssembly(): string {
        let assembly = '';

        const root = this.ast.getRoot();
        const children = root.getChildren();
        children.forEach((c) => {
            assembly += this.genCodeForAdd(c as AddASTNode);
        });

        return assembly;
    }

    public genCodeForAdd(ast: AddASTNode): string {
        const [left, right] = ast.getChildren();
        let leftCode = '';

        console.log(left.getType());
        if (left.getType() == Token.PLUS) {
            leftCode = this.genCodeForAdd(left as AddASTNode);
        }

        if (left.getType() == Token.NUMBER) {
            leftCode = `mov eax, ${left.getValue()}\n`;
        }

        let rightCode = '';
        if (right instanceof AddASTNode) {
            rightCode = this.genCodeForAdd(right);
        }

        if (right instanceof NumberASTNode) {
            rightCode = `add eax, ${right.getValue()}\n`;
        }

        if (leftCode === '' || rightCode === '') {
            throw new CompilerError('Invalid AST');
        }

        return `${leftCode}${rightCode}`;
    }
}
