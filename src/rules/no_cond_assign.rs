// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule};
use swc_common::Span;
use swc_ecma_ast::Expr::{self, Assign, Paren};
use swc_ecma_ast::Module;
use swc_ecma_ast::{
  ParenExpr,
  Stmt::{self, DoWhile, For, If, While},
};
use swc_ecma_visit::{Node, Visit};

pub struct NoCondAssign;

impl LintRule for NoCondAssign {
  fn new() -> Box<Self> {
    Box::new(NoCondAssign)
  }

  fn lint_module(&self, context: Context, module: Module) {
    let mut visitor = NoCondAssignVisitor::new(context);
    visitor.visit_module(&module, &module);
  }
}

pub struct NoCondAssignVisitor {
  context: Context,
}

impl NoCondAssignVisitor {
  pub fn new(context: Context) -> Self {
    Self { context }
  }

  fn add_diagnostic(&self, span: Span) {
    self.context.add_diagnostic(
      span,
      "noCondAssign",
      "Expected a conditional expression and instead saw an assignment",
    );
  }

  fn check_test_expr(&self, expr: &Expr) {
    match expr {
      Assign(assign) => self.add_diagnostic(assign.span),
      Paren(paren) => self.check_paren_expr(&paren),
      _ => {}
    }
  }

  fn check_paren_expr(&self, paren_expr: &ParenExpr) {
    match paren_expr.expr.as_ref() {
      Assign(assign) => self.add_diagnostic(assign.span),
      Paren(paren_expr) => self.check_paren_expr(paren_expr),
      _ => {}
    }
  }
}

impl Visit for NoCondAssignVisitor {
  fn visit_stmt(&mut self, stmt: &Stmt, _parent: &dyn Node) {
    match stmt {
      If(if_stmt) => self.check_test_expr(if_stmt.test.as_ref()),
      While(while_stmt) => self.check_test_expr(while_stmt.test.as_ref()),
      DoWhile(do_while) => self.check_test_expr(&*do_while.test.as_ref()),
      For(for_stmt) => {
        if let Some(test) = for_stmt.test.as_deref() {
          self.check_test_expr(test);
        }
      }
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::test_lint;
  use serde_json::json;

  #[test]
  fn it_passes_using_equality_operator() {
    test_lint(
      "no_cond_assign",
      r#"
if (x === 0) {
}
     "#,
      vec![NoCondAssign::new()],
      json!([]),
    )
  }

  #[test]
  fn it_passes_with_bracketed_assignment() {
    test_lint(
      "no_cond_assign",
      r#"
if ((x = y)) {
}
     "#,
      vec![NoCondAssign::new()],
      json!([]),
    )
  }

  #[test]
  fn it_fails_using_assignment_in_if_stmt() {
    test_lint(
      "no_cond_assign",
      r#"
if (x = 0) {
}
      "#,
      vec![NoCondAssign::new()],
      json!([{
        "code": "noCondAssign",
        "message": "Expected a conditional expression and instead saw an assignment",
        "location": {
          "filename": "no_cond_assign",
          "line": 2,
          "col": 4,
        }
      }]),
    )
  }

  #[test]
  fn it_fails_using_assignment_in_while_stmt() {
    test_lint(
      "no_cond_assign",
      r#"
while (x = 0) {
}
      "#,
      vec![NoCondAssign::new()],
      json!([{
        "code": "noCondAssign",
        "message": "Expected a conditional expression and instead saw an assignment",
        "location": {
          "filename": "no_cond_assign",
          "line": 2,
          "col": 7,
        }
      }]),
    )
  }

  #[test]
  fn it_fails_using_assignment_in_do_while_stmt() {
    test_lint(
      "no_cond_assign",
      r#"
do {
} while (x = 0);
      "#,
      vec![NoCondAssign::new()],
      json!([{
        "code": "noCondAssign",
        "message": "Expected a conditional expression and instead saw an assignment",
        "location": {
          "filename": "no_cond_assign",
          "line": 3,
          "col": 9,
        }
      }]),
    )
  }

  #[test]
  fn it_fails_using_assignment_in_for_stmt() {
    test_lint(
      "no_cond_assign",
      r#"
for (let i = 0; i = 10; i++) {
}
      "#,
      vec![NoCondAssign::new()],
      json!([{
        "code": "noCondAssign",
        "message": "Expected a conditional expression and instead saw an assignment",
        "location": {
          "filename": "no_cond_assign",
          "line": 2,
          "col": 16,
        }
      }]),
    )
  }
}
