use inkwell::{builder::Builder, context::Context, module::Module};

use crate::{
    ast::{Assignment, Declaration, IfStatement, Statement, StatementOrExpression, WhileLoop},
    symbol_table::FunctionDefinition,
};

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("main"),
            builder: context.create_builder(),
        }
    }

    pub fn compile_program(&mut self, _program: &crate::ast::Program) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let main_fn = self
            .module
            .add_function("main", i32_type.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);
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
            StatementOrExpression::Expression(expression) => self.compile_expression(expression),
        }
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Declaration(declaration) => self.compile_declaration(declaration),
            Statement::Assign(assign) => self.compile_assign(assign),
            Statement::WhileLoop(while_loop) => self.compile_while_loop(while_loop),
            Statement::IfStatement(if_stmt) => self.compile_if_statement(if_stmt),
            Statement::FunctionDefintion(func_def) => self.compile_function_definition(func_def),
            Statement::Return(expr) => self.compile_expression(expr),
        }
    }

    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<(), String> {
        // Register the variable
    }

    fn compile_assign(&mut self, assign: &Assignment) -> Result<(), String> {
        todo!();
    }

    fn compile_while_loop(&mut self, _while_loop: &WhileLoop) -> Result<(), String> {
        todo!();
    }

    fn compile_if_statement(&mut self, _if_stmt: &IfStatement) -> Result<(), String> {
        todo!();
    }

    fn compile_function_definition(
        &mut self,
        _func_def: &FunctionDefinition,
    ) -> Result<(), String> {
        todo!();
    }
}
