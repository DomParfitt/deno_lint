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
      for stmt in after {
        self.context.add_diagnostic(
          stmt.span(),
          "noUnreachable",
          "Unreachable code",
        )
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
  fn it_passes() {
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
  fn it_fails() {
    test_lint(
      "no_unreachable",
      r#"
function foo() {
  return true;
  console.log("done");
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
