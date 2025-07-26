import { Program } from './AST';
import Scope from './Scope';

export default class Environment {
    private globalScope: Scope;

    constructor(
        private ast: Program,
        private debug: boolean = false,
    ) {
        this.globalScope = new Scope(null, null);
    }

    public getGlobalScope(): Scope {
        return this.globalScope;
    }
}
