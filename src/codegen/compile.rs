use crate::{
    ast::{
        AtomType, BinaryOp, Block, Expression, ExpressionKind, Program,
        StatementOrExpression::{self},
        UnaryOp,
    },
    codegen::{
        Codegen, CodegenError,
        address::{Address, Register},
        compile_call::compile_function_call,
        compile_memory::{
            compile_struct_instantiation, load_element, mem_copy, with_place_address,
        },
        compile_operators::{
            compile_binary, compile_cast, compile_logical_binary, compile_postfix,
            compile_postfix_update, compile_unary,
        },
        data_name,
        instruction::{Instruction, Operand},
        symbol_table::SymbolTable,
    },
    error::ResultAt,
    type_check::Type,
    types::{NumberType, Span},
};

impl Program {
    pub fn compile(&self) -> Result<String, CodegenError> {
        let mut codegen = Codegen::default();

        let mut symbol_table = SymbolTable::default();
        self.block.discover(&mut symbol_table);
        symbol_table.externs.insert("exit".to_string());
        codegen.defined_functions = symbol_table.functions.keys().cloned().collect();

        let instructions = render(&self.block.compile(&mut codegen)?);

        let functions = symbol_table
            .functions
            .values()
            .map(|def| {
                def.compile(&mut codegen)
                    .map(|instructions| render(&instructions))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        let globals = symbol_table
            .functions
            .keys()
            .map(|name| format!("global {name}"))
            .collect::<Vec<_>>()
            .join("\n");

        let extern_functions = symbol_table
            .externs
            .iter()
            .map(|e| format!("extern {e}"))
            .collect::<Vec<_>>()
            .join("\n");

        let data = codegen
            .data
            .iter()
            .enumerate()
            .map(|(i, d)| format!("\t{}", format_data(&data_name(i), d)))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(format!(
            "BITS 64

global main
{globals}
{extern_functions}

SECTION .rdata
{data}

SECTION .text

{functions}

main:
\tpush rbp
\tmov rbp, rsp
{instructions}
\tmov rsp, rbp
\tpop rbp
\tsub rsp, 40
\txor rcx, rcx
\tcall exit"
        ))
    }
}

pub fn render(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

impl Block {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let outer_depth = codegen.stack_depth();
        codegen.with_block(|codegen| {
            let mut instructions = self
                .children
                .iter()
                .map(|child| child.compile(codegen))
                .collect::<Result<Vec<_>, CodegenError>>()?
                .concat();

            let alloc_size = codegen.stack_depth() - outer_depth;
            // A trailing return, break or continue jumps away, so a dealloc after it is unreachable
            if alloc_size > 0 && !matches!(instructions.last(), Some(Instruction::Jmp { .. })) {
                instructions.push(Instruction::add(Register::RSP, alloc_size));
            }

            Ok(instructions)
        })
    }
}

impl StatementOrExpression {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        match &self {
            StatementOrExpression::Statement(statement) => statement.compile(codegen),
            StatementOrExpression::Expression(expression) => Ok([
                vec![Instruction::comment(expression.to_string())],
                expression.compile_discarded(codegen)?,
            ]
            .concat()),
        }
    }
}

impl Expression {
    pub fn compile(
        &self,
        codegen: &mut Codegen,
        dst: impl Into<Address>,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let dst = dst.into();

        match &self.kind {
            ExpressionKind::Atom(a) => match a {
                AtomType::Number(n) => compile_number(*n, dst, codegen),
                AtomType::Char(c) => Ok(vec![compile_char(*c, dst)]),
                AtomType::String(s) => compile_string(s, dst, codegen),
                AtomType::Boolean(b) => Ok(vec![compile_boolean(*b, dst)]),
                AtomType::Null => Ok(vec![Instruction::mov(dst, NumberType::I64(0))]),
            },
            ExpressionKind::Variable { name, .. } => {
                compile_variable(codegen, name, self.etype(), self.span, dst)
            }
            ExpressionKind::Binary(BinaryOp::And | BinaryOp::Or, ..) => {
                compile_logical_binary(self, codegen, dst)
            }
            ExpressionKind::Binary(op, left, right) => {
                compile_binary(*op, left, right, self.etype(), codegen, dst)
            }
            ExpressionKind::Index(..)
            | ExpressionKind::StructAccess { .. }
            | ExpressionKind::Unary(UnaryOp::Deref, _) => {
                with_place_address(self, codegen, &mut |codegen, address, t| match t {
                    Type::Struct(struct_def) => mem_copy(struct_def, dst, address, codegen),
                    _ => move_into(dst, codegen, |reg| load_element(t, reg, address)),
                })
            }
            ExpressionKind::Unary(op, child) => compile_unary(*op, child, codegen, dst),
            ExpressionKind::Postfix(op, lhs) => compile_postfix(*op, lhs, codegen, dst),
            ExpressionKind::FunctionCall(name, args) => {
                compile_function_call(codegen, name, args, self.etype(), Some(dst))
            }
            ExpressionKind::Cast(inner, cast_type) => compile_cast(inner, cast_type, codegen, dst),
            ExpressionKind::SizeOf(sized_type) => compile_number(
                NumberType::Coercible(u64::try_from(sized_type.size()).unwrap()),
                dst,
                codegen,
            ),
            ExpressionKind::StructInstantiation { fields, .. } => {
                compile_struct_instantiation(self.etype(), fields, codegen, dst)
            }
            ExpressionKind::EnumVariant { variant_name, .. } => {
                compile_enum(variant_name, self.etype(), codegen, dst)
            }
        }
    }

    // For when the output is not directly read. mostly used as a side affect like a postfix
    pub fn compile_discarded(
        &self,
        codegen: &mut Codegen,
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self.kind {
            ExpressionKind::Postfix(op, lhs) => compile_postfix_update(*op, lhs, codegen),
            ExpressionKind::FunctionCall(name, args) => {
                compile_function_call(codegen, name, args, self.etype(), None)
            }
            ExpressionKind::Unary(_, child) | ExpressionKind::Cast(child, _) => {
                child.compile_discarded(codegen)
            }
            ExpressionKind::Binary(BinaryOp::Divide | BinaryOp::Modulo, ..) => {
                codegen.lease_with_scope(|codegen, dst| self.compile(codegen, dst))
            }
            ExpressionKind::Binary(_, left, right) | ExpressionKind::Index(left, right) => {
                let mut instructions = left.compile_discarded(codegen)?;
                instructions.extend(right.compile_discarded(codegen)?);
                Ok(instructions)
            }
            ExpressionKind::Atom(_)
            | ExpressionKind::Variable { .. }
            | ExpressionKind::SizeOf(_)
            | ExpressionKind::StructInstantiation { .. }
            | ExpressionKind::StructAccess { .. }
            | ExpressionKind::EnumVariant { .. } => Ok(vec![]),
        }
    }
}

fn compile_enum(
    variant_name: &str,
    enum_type: &Type,
    codegen: &mut Codegen,
    dst: Address,
) -> Result<Vec<Instruction>, CodegenError> {
    let index = variant_index(enum_type, variant_name).unwrap();
    compile_number(NumberType::U8(index), dst, codegen)
}

fn variant_index(enum_type: &Type, variant_name: &str) -> Option<u8> {
    let Type::Enum(enum_def) = enum_type else {
        return None;
    };
    u8::try_from(enum_def.variant_index(variant_name)?).ok()
}

fn compile_string(
    s: &str,
    dst: Address,
    codegen: &mut Codegen,
) -> Result<Vec<Instruction>, CodegenError> {
    let handle = codegen.add_data(s);
    move_into(dst, codegen, |reg| {
        Instruction::mov(reg, Operand::Label(handle))
    })
}

pub fn move_into(
    dst: Address,
    codegen: &mut Codegen,
    instruction: impl FnOnce(Register) -> Instruction,
) -> Result<Vec<Instruction>, CodegenError> {
    match dst {
        Address::Register(reg) => Ok(vec![instruction(reg)]),
        _ => codegen
            .lease_with_scope(|_, reg| Ok(vec![instruction(reg), Instruction::mov(dst, reg)])),
    }
}

fn compile_number(
    n: NumberType,
    dst: Address,
    codegen: &mut Codegen,
) -> Result<Vec<Instruction>, CodegenError> {
    if fits_imm32(n) {
        Ok(vec![Instruction::mov(dst, n)])
    } else {
        move_into(dst, codegen, |reg| Instruction::mov(reg, n))
    }
}

fn compile_char(c: char, dst: Address) -> Instruction {
    Instruction::mov(dst, c)
}

fn compile_variable(
    codegen: &mut Codegen,
    name: &str,
    t: &Type,
    span: Span,
    dst: Address,
) -> Result<Vec<Instruction>, CodegenError> {
    let address = codegen.get_variable(name).at(span)?;
    if dst == address {
        return Ok(vec![]);
    }

    if let Type::Struct(struct_def) = t {
        return mem_copy(struct_def, dst, address, codegen);
    }

    match address {
        Address::Register(_) => Ok(vec![Instruction::mov(dst, address)]),
        _ => move_into(dst, codegen, |reg| Instruction::mov(reg, address)),
    }
}

fn compile_boolean(b: bool, dst: Address) -> Instruction {
    let n = NumberType::I64(i64::from(b));
    Instruction::mov(dst, n)
}

pub fn as_operand(expr: &Expression, codegen: &Codegen) -> Option<Operand> {
    match &expr.kind {
        ExpressionKind::Atom(AtomType::Number(n)) if fits_imm32(*n) => Some(Operand::Immediate(*n)),
        ExpressionKind::Atom(AtomType::Char(c)) => Some(Operand::from(*c)),
        ExpressionKind::Variable { name, .. } => codegen.get_variable(name).ok().map(Operand::from),
        ExpressionKind::EnumVariant { variant_name, .. } => {
            variant_index(expr.etype(), variant_name)
                .map(|index| Operand::Immediate(NumberType::U8(index)))
        }
        _ => None,
    }
}

fn fits_imm32(n: NumberType) -> bool {
    match n {
        NumberType::U8(_) | NumberType::I32(_) => true,
        NumberType::U32(n) => i32::try_from(n).is_ok(),
        NumberType::U64(n) | NumberType::Coercible(n) => i32::try_from(n).is_ok(),
        NumberType::I64(n) => i32::try_from(n).is_ok(),
    }
}

fn format_data(name: &str, s: &str) -> String {
    let mut parts = vec![];
    let mut run = String::new();
    for c in s.chars() {
        if c == ' ' || (c.is_ascii_graphic() && c != '"') {
            run.push(c);
            continue;
        }

        if !run.is_empty() {
            parts.push(format!("\"{run}\""));
            run.clear();
        }

        let mut buf = [0; 4];
        parts.extend(c.encode_utf8(&mut buf).bytes().map(|b| b.to_string()));
    }

    if !run.is_empty() {
        parts.push(format!("\"{run}\""));
    }
    parts.push("0".to_string());

    format!("{name} db {}", parts.join(", "))
}

#[cfg(test)]
pub mod test {
    use crate::{lex::lex, parse::parse, type_check::check_types};

    pub fn compile(input: &str) -> String {
        let mut ast = parse(lex(input, 0).unwrap()).unwrap();
        check_types(&mut ast).unwrap();
        ast.compile().unwrap()
    }

    #[test]
    fn compile_boolean_literals() {
        let asm = compile("let a = true; let b = false;");
        assert!(asm.contains("mov QWORD [rbp - 8], 1"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 0"), "{asm}");
    }

    #[test]
    fn compile_discarded_postfix_skips_old_value() {
        let asm = compile("let i = 0; i++;");
        assert!(!asm.contains(", [rbp - 8]"), "{asm}");
    }

    #[test]
    fn compile_discarded_call_keeps_result_in_rax() {
        let asm = compile("def five(): i32 { 5 } five();");
        assert!(!asm.contains(", rax"), "{asm}");
    }

    #[test]
    fn compile_declared_extern_call() {
        let asm = compile("extern def puts(s: *char): i32; let n = puts(\"hi\");");
        assert!(asm.contains("extern puts"), "{asm}");
        assert!(asm.contains("call puts"), "{asm}");
    }

    #[test]
    fn compile_boolean_is_not_a_variable_lookup() {
        compile("if (true) { printf(\"yes\"); }");
        compile("let a = !false;");
    }

    #[test]
    fn compile_char_literals() {
        let asm = compile("let a: char = 'a'; let b: char = 'Z'; let c: char = '0';");
        assert!(asm.contains("mov QWORD [rbp - 8], 97"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 90"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 24], 48"), "{asm}");
        assert!(!asm.contains("db "), "{asm}");
    }

    #[test]
    fn compile_char_escapes() {
        let asm = compile(r"let a: char = '\n'; let b: char = '\0'; let c: char = '\'';");
        assert!(asm.contains("mov QWORD [rbp - 8], 10"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 0"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 24], 39"), "{asm}");
    }

    #[test]
    fn compile_char_arithmetic() {
        compile("let c: char = '7'; let d = c - '0';");
        compile("let c: char = 'a'; c += 1;");
        compile("let c: char = 'a'; c++;");
    }

    #[test]
    fn compile_char_comparison() {
        compile("let c: char = 'a'; if (c == 'a') { printf(\"yes\"); }");
        compile("let c: char = 'a'; let lower = c >= 'a' && c <= 'z';");
    }

    #[test]
    fn compile_char_as_function_arg() {
        let asm = compile("putchar('h');");
        assert!(asm.contains("104"), "{asm}");
        compile("printf(\"%c%c\", 'h', 'i');");
    }

    #[test]
    fn compile_char_function_param_and_return() {
        compile("def next(c: char): char { c + 1 } next('a');");
    }

    #[test]
    fn compile_large_literal_to_stack_goes_through_register() {
        let asm = compile("let big: u32 = 4000000000;");
        assert!(!asm.contains("], 4000000000"), "{asm}");
        assert!(asm.contains(", 4000000000"), "{asm}");
    }

    #[test]
    fn compile_small_literal_to_stack_is_immediate() {
        let asm = compile("let small: u32 = 5;");
        assert!(asm.contains("], 5"), "{asm}");
    }
}
