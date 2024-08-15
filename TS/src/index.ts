import util from 'util';
import Environment from './Environment';
import Lexer from './Lexer';
import Optimizer from './Optimizer';
import Parser from './Parser';
import { readScript, writeInstructions } from './util';

const DEBUG = false;

const script = readScript('script.tan');
const lexer = new Lexer(script);
const tokens = lexer.getTokens();
if (DEBUG) {
    console.log('Tokens:', tokens);
}

const parser = new Parser(tokens);
let ast = parser.parse();

const instructions = ast.compile();
if (DEBUG) {
    console.log('Instructions:', instructions);
}

writeInstructions(instructions);

ast = Optimizer.optimize(ast);
if (DEBUG) {
    console.log(util.inspect(ast, { showHidden: false, depth: null }));
}

const env = new Environment(ast, true);

console.time('Execution time');
env.evaluate();
console.timeEnd('Execution time');
