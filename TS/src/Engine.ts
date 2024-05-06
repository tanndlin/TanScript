import Environment from './Environment';
import Lexer from './Lexer';
import Parser from './Parser';

export default class Engine {
    private lexer: Lexer;
    private parser: Parser;
    private environment: Environment;

    constructor(private script: string) {
        this.lexer = new Lexer(script);
        const tokens = this.lexer.getTokens();

        this.parser = new Parser(tokens);
        const ast = this.parser.parse();

        this.environment = new Environment(ast);
    }

    public run() {
        return this.environment.evaluate();
    }

    public getEnvironment() {
        return this.environment;
    }
}
