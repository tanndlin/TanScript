import { AST, RootASTNode } from '../AST/AST';
import { AddASTNode, NumberASTNode } from '../AST/NumberAST';
import Compiler from '../Compiler';

describe('Compiler Test', () => {
    it('should compile the AST into assembly code', () => {
        const root = new RootASTNode([
            new AddASTNode(new NumberASTNode('10'), new NumberASTNode('20')),
            new AddASTNode(new NumberASTNode('30'), new NumberASTNode('40')),
        ]);

        const compiler = new Compiler(new AST(root));
        const assembly = compiler.generateAssembly();

        expect(assembly).toBe(
            'mov eax, 10\nadd eax, 20\nmov eax, 30\nadd eax, 40\n'
        );
    });
});
