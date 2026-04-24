use std::collections::HashMap;

use inkwell::{
    AddressSpace, IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue},
};

use crate::{
    ast::{
        Assignment, AtomType, Declaration, Expression, IfStatement, OperatorType, Program,
        Statement, StatementOrExpression, WhileLoop,
    },
    symbol_table::FunctionDefinition,
};

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("main"),
            builder: context.create_builder(),
            variables: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let main_fn = self
            .module
            .add_function("main", i32_type.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        // Compile the program's block
        for stmt_or_expr in &program.block.children {
            self.compile_statement_or_expression(stmt_or_expr)?;
        }

        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn compile_statement_or_expression(
        &mut self,
        stmt_or_expr: &StatementOrExpression,
    ) -> Result<(), String> {
        match stmt_or_expr {
            StatementOrExpression::Statement(statement) => self.compile_statement(statement),
            StatementOrExpression::Expression(expression) => {
                self.compile_expression(expression).map(|_| ())
            }
        }
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Declaration(declaration) => self.compile_declaration(declaration),
            Statement::Assign(assign) => self.compile_assign(assign),
            Statement::WhileLoop(while_loop) => self.compile_while_loop(while_loop),
            Statement::IfStatement(if_stmt) => self.compile_if_statement(if_stmt),
            Statement::FunctionDefintion(func_def) => self.compile_function_definition(func_def),
            Statement::Return(expr) => {
                let value = self.compile_expression(expr)?;
                self.builder
                    .build_return(Some(&value))
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
        }
    }

    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<(), String> {
        self.compile_assign(&declaration.assign)
    }

    fn compile_assign(&mut self, assign: &Assignment) -> Result<(), String> {
        let value = self.compile_expression(&assign.expression)?;
        let ptr = match self.variables.get(&assign.identifier).copied() {
            Some((ptr, _)) => ptr,
            None => {
                let ptr = self
                    .builder
                    .build_alloca(value.get_type(), &assign.identifier)
                    .map_err(|e| e.to_string())?;
                self.variables
                    .insert(assign.identifier.clone(), (ptr, value.get_type()));
                ptr
            }
        };
        self.builder
            .build_store(ptr, value)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::Atom(atom) => self.compile_atom(atom),
            Expression::Operation(op, operands) => self.compile_operation(op, operands),
            Expression::FunctionCall(name, args) => self.compile_function_call(name, args),
        }
    }

    fn compile_atom(&mut self, atom: &AtomType) -> Result<BasicValueEnum<'ctx>, String> {
        match atom {
            AtomType::Number(n) => Ok(self.context.i32_type().const_int(*n as u64, true).into()),
            AtomType::Identifier(name) => {
                let (ptr, ty) = self
                    .variables
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("Undefined variable: {name}"))?;
                self.builder
                    .build_load(ty, ptr, name)
                    .map_err(|e| e.to_string())
            }
            AtomType::String(s) => Ok(self
                .builder
                .build_global_string_ptr(s, "str")
                .map_err(|e| e.to_string())?
                .as_pointer_value()
                .into()),
        }
    }

    fn compile_operation(
        &mut self,
        op: &OperatorType,
        operands: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if *op == OperatorType::Not {
            let val = self.compile_expression(&operands[0])?.into_int_value();
            return self
                .builder
                .build_not(val, "not")
                .map(Into::into)
                .map_err(|e| e.to_string());
        }

        let lhs = self.compile_expression(&operands[0])?.into_int_value();
        let rhs = self.compile_expression(&operands[1])?.into_int_value();

        match op {
            OperatorType::Add => self.builder.build_int_add(lhs, rhs, "add"),
            OperatorType::Subtract => self.builder.build_int_sub(lhs, rhs, "sub"),
            OperatorType::Multiply => self.builder.build_int_mul(lhs, rhs, "mul"),
            OperatorType::Divide => self.builder.build_int_signed_div(lhs, rhs, "div"),
            OperatorType::Modulo => self.builder.build_int_signed_rem(lhs, rhs, "rem"),
            OperatorType::LessThan => {
                self.builder
                    .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt")
            }
            OperatorType::LessOrEqual => {
                self.builder
                    .build_int_compare(IntPredicate::SLE, lhs, rhs, "le")
            }
            OperatorType::GreaterThan => {
                self.builder
                    .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt")
            }
            OperatorType::GreaterOrEqual => {
                self.builder
                    .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge")
            }
            OperatorType::Equal => self
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
            OperatorType::NotEqual => {
                self.builder
                    .build_int_compare(IntPredicate::NE, lhs, rhs, "ne")
            }
            OperatorType::Or => self.builder.build_or(lhs, rhs, "or"),
            OperatorType::And => self.builder.build_and(lhs, rhs, "and"),
            other => return Err(format!("Unsupported operator in expression: {other}")),
        }
        .map(Into::into)
        .map_err(|e| e.to_string())
    }

    fn compile_function_call(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = args
            .iter()
            .map(|arg| self.compile_expression(arg).map(Into::into))
            .collect::<Result<_, _>>()?;

        let function = self.module.get_function(name).unwrap_or_else(|| {
            let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            self.module
                .add_function(name, ptr_type.fn_type(&[], true), None)
        });

        let call = self
            .builder
            .build_call(function, &compiled_args, name)
            .map_err(|e| e.to_string())?;

        // If the function returns void, yield a null ptr so callers have a BasicValueEnum
        Ok(call.try_as_basic_value().left().unwrap_or_else(|| {
            self.context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .const_null()
                .into()
        }))
    }

    fn compile_while_loop(&mut self, _while_loop: &WhileLoop) -> Result<(), String> {
        todo!()
    }

    fn compile_if_statement(&mut self, _if_stmt: &IfStatement) -> Result<(), String> {
        todo!()
    }

    fn compile_function_definition(
        &mut self,
        _func_def: &FunctionDefinition,
    ) -> Result<(), String> {
        todo!()
    }
}
