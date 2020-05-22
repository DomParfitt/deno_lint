// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule};
use swc_common::Spanned;
use swc_ecma_ast::Stmt::{self, Break, Continue, Return, Throw};
use swc_ecma_ast::{BlockStmt, Module};
use swc_ecma_visit::{Node, Visit};

pub struct NoUnreachable;

impl LintRule for NoUnreachable {
  fn new() -> Box<Self> {
    Box::new(NoUnreachable)
  }

  fn lint_module(&self, context: Context, module: Module) {
    let mut visitor = NoUnreachableVisitor::new(context);
    visitor.visit_module(&module, &module);
  }
}

pub struct NoUnreachableVisitor {
  context: Context,
}

impl NoUnreachableVisitor {
  pub fn new(context: Context) -> Self {
    Self { context }
  }
}

impl Visit for NoUnreachableVisitor {
  fn visit_block_stmt(&mut self, block_stmt: &BlockStmt, _parent: &dyn Node) {
    if let Some((idx, _)) = block_stmt
      .stmts
      .iter()
      .enumerate()
      .find(|(_, stmt)| is_return(stmt))
    {
      let (_, after) = block_stmt.stmts.split_at(idx);
      for (i, stmt) in after.iter().enumerate() {
        if i == idx {
          continue;
        }

        self.context.add_diagnostic(
          stmt.span(),
          "noUnreachable",
          "Unreachable code",
        );
      }
    }
  }
}

fn is_return(stmt: &Stmt) -> bool {
  match stmt {
    Return(_) | Break(_) | Continue(_) | Throw(_) => true,
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::test_lint;
  use serde_json::json;

  #[test]
  fn it_passes_when_there_is_no_unreachable_code_after_a_return() {
    test_lint(
      "no_unreachable",
      r#"
function foo() {
  return;
}
      "#,
      vec![NoUnreachable::new()],
      json!([]),
    )
  }

  #[test]
  fn it_passes_when_there_is_no_unreachable_code_after_a_throw() {
    test_lint(
      "no_unreachable",
      r#"
function foo() {
  throw new Error();
}
      "#,
      vec![NoUnreachable::new()],
      json!([]),
    )
  }

  #[test]
  fn it_passes_when_there_is_no_unreachable_code_after_a_break() {
    test_lint(
      "no_unreachable",
      r#"
while(value) {
  break;
}
      "#,
      vec![NoUnreachable::new()],
      json!([]),
    )
  }

  #[test]
  fn it_passes_when_there_is_no_unreachable_code_after_a_continue() {
    test_lint(
      "no_unreachable",
      r#"
for (var i = 0; i < 10; i++) {
  continue;
}
      "#,
      vec![NoUnreachable::new()],
      json!([]),
    )
  }

  #[test]
  fn it_passes_with_function_hoisting() {
    test_lint(
      "no_unreachable",
      r#"
function foo() {
  return bar();
  function bar() {
      return 1;
  }
}
      "#,
      vec![NoUnreachable::new()],
      json!([]),
    )
  }

  #[test]
  fn it_passes_with_variable_hoisting() {
    test_lint(
      "no_unreachable",
      r#"
function bar() {
  return x;
  var x;
}
      "#,
      vec![NoUnreachable::new()],
      json!([]),
    )
  }

  #[test]
  fn it_fails_when_there_is_unreachable_code_after_a_return() {
    test_lint(
      "no_unreachable",
      r#"
function foo() {
  return;
  console.log();
}
      "#,
      vec![NoUnreachable::new()],
      json!([{
        "code": "noUnreachable",
        "message": "Unreachable code",
        "location": {
          "filename": "no_unreachable",
          "line": 4,
          "col": 2,
        }
      }]),
    )
  }

  #[test]
  fn it_fails_when_there_is_unreachable_code_after_a_throw() {
    test_lint(
      "no_unreachable",
      r#"
function foo() {
  throw new Error();
  console.log();
}
      "#,
      vec![NoUnreachable::new()],
      json!([{
        "code": "noUnreachable",
        "message": "Unreachable code",
        "location": {
          "filename": "no_unreachable",
          "line": 4,
          "col": 2,
        }
      }]),
    )
  }

  #[test]
  fn it_fails_when_there_is_unreachable_code_after_a_break() {
    test_lint(
      "no_unreachable",
      r#"
while(value) {
  break;
  console.log();
}
      "#,
      vec![NoUnreachable::new()],
      json!([{
        "code": "noUnreachable",
        "message": "Unreachable code",
        "location": {
          "filename": "no_unreachable",
          "line": 4,
          "col": 2,
        }
      }]),
    )
  }

  #[test]
  fn it_fails_when_there_is_unreachable_code_after_a_continue() {
    test_lint(
      "no_unreachable",
      r#"
for (var i = 0; i < 10; i++) {
  continue;
  console.log();
}
      "#,
      vec![NoUnreachable::new()],
      json!([{
        "code": "noUnreachable",
        "message": "Unreachable code",
        "location": {
          "filename": "no_unreachable",
          "line": 4,
          "col": 2,
        }
      }]),
    )
  }
}
