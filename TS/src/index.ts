import util from 'util';
import Environment from './Environment';
import Lexer from './Lexer';
import Optimizer from './Optimizer';
import Parser from './Parser';
import { readScript, writeInstructions } from './util';

const script = readScript('script.tan');
const lexer = new Lexer(script);
const tokens = lexer.getTokens();

console.log('Tokens:', tokens);

const parser = new Parser(tokens);
let ast = parser.parse();

const instructions = ast.compile();
console.log('Instructions:', instructions);
writeInstructions(instructions);

ast = Optimizer.optimize(ast);

console.log(util.inspect(ast, { showHidden: false, depth: null }));

const env = new Environment(ast, true);

console.time('Execution time');
env.evaluate();
console.timeEnd('Execution time');
