import { AST } from './AST/AST';
import Scope from './Scope';
import { RuntimeValue, Token } from './types';

export default class Environment {
    private globalScope: Scope;

    constructor(private ast: AST, private debug: boolean = false) {
        this.globalScope = new Scope(null);
    }

    public evaluate(): RuntimeValue {
        let retValue: RuntimeValue = null;

        const root = this.ast.getRoot();
        const statements = root.getChildren();
        statements.forEach((statement, i) => {
            if (statement.isType(Token.EOF)) return;

            if (this.debug) console.log(`Statement ${i + 1}:`);
            retValue = statement.evaluate(this.globalScope);

            if (this.debug) console.log(retValue);
        });

        return retValue;
    }

    public getGlobalScope(): Scope {
        return this.globalScope;
    }
}
