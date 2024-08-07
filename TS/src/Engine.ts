import Environment from './Environment';
import Lexer from './Lexer';
import Optimizer from './Optimizer';
import Parser from './Parser';

export default class Engine {
    private lexer: Lexer;

    private parser: Parser;

    private environment: Environment;

    constructor(private script: string) {
        this.lexer = new Lexer(script);
        const tokens = this.lexer.getTokens();

        this.parser = new Parser(tokens);
        let ast = this.parser.parse();
        ast = Optimizer.optimize(ast);

        this.environment = new Environment(ast);
    }

    public run() {
        return this.environment.evaluate();
    }

    public getEnvironment() {
        return this.environment;
    }
}
