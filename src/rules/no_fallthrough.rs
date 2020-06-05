use super::{Context, LintRule};
use swc_ecma_ast::Stmt::{Break, Continue, Return, Throw};
use swc_ecma_ast::{Module, Stmt, SwitchStmt};
use swc_ecma_visit::{Node, Visit};

pub struct NoFallthrough;

impl LintRule for NoFallthrough {
  fn new() -> Box<Self> {
    Box::new(NoFallthrough)
  }

  fn lint_module(&self, context: Context, module: Module) {
    let mut visitor = NoFallthroughVisitor::new(context);
    visitor.visit_module(&module, &module);
  }
}

pub struct NoFallthroughVisitor {
  context: Context,
}

impl NoFallthroughVisitor {
  pub fn new(context: Context) -> Self {
    Self { context }
  }
}

impl Visit for NoFallthroughVisitor {
  fn visit_switch_stmt(&mut self, switch: &SwitchStmt, _parent: &dyn Node) {
    for case in &switch.cases {
      if case.cons.iter().any(|stmt| is_control_flow_stmt(stmt)) {
        continue;
      }

      self.context.add_diagnostic(
        case.span,
        "noFallthrough",
        "Expected break statement before case",
      )
    }
  }
}

fn is_control_flow_stmt(stmt: &Stmt) -> bool {
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
  fn it_passes_for_a_switch_with_no_fallthrough() {
    test_lint(
      "no_fallthrough",
      r#"
switch(foo) {
  case 1:
    doSomething();
    break;

  case 2:
    doSomething();
}
      "#,
      vec![NoFallthrough::new()],
      json!([]),
    )
  }
}
