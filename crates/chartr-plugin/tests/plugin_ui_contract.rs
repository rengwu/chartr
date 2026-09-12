//! Every bundled plugin, including newly added directories, shares the same UI
//! primitives. Custom painting belongs in reviewed SDK components, not plugins.

use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};

fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr.parse_args::<syn::Path>().is_ok_and(|path| path.is_ident("test"))
    })
}

#[derive(Default)]
struct Contract {
    problems: Vec<String>,
}

impl<'ast> Visit<'ast> for Contract {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if !test_only(&node.attrs) {
            visit::visit_item_mod(self, node);
        }
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if !test_only(&node.attrs) {
            visit::visit_item_fn(self, node);
        }
    }
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        if matches!(method.as_str(), "bg" | "border_color")
            || method.starts_with("rounded_")
            || method.starts_with("shadow_")
        {
            self.problems
                .push(format!(".{method}(): use or extend chartr_plugin::ui's semantic surface"));
        }
        visit::visit_expr_method_call(self, node);
    }
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            let segments: Vec<_> =
                path.path.segments.iter().map(|segment| segment.ident.to_string()).collect();
            if segments.len() >= 2
                && segments.last().unwrap() == "new"
                && matches!(
                    segments[segments.len() - 2].as_str(),
                    "Label" | "Button" | "IconButton"
                )
            {
                self.problems.push(format!(
                    "{}: use semantic labels/actions from chartr_plugin::ui",
                    segments.join("::")
                ));
            }
        }
        visit::visit_expr_call(self, node);
    }
}

fn rust_files(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

#[test]
fn all_plugin_presentation_uses_shared_components() {
    let mut paths = Vec::new();
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins"), &mut paths);
    let mut problems = Vec::new();
    for path in paths {
        let mut contract = Contract::default();
        contract.visit_file(&syn::parse_file(&std::fs::read_to_string(&path).unwrap()).unwrap());
        problems.extend(
            contract.problems.into_iter().map(|problem| format!("{}: {problem}", path.display())),
        );
    }
    assert!(problems.is_empty(), "Plugin UI drift:\n{}", problems.join("\n"));
}

#[test]
fn the_contract_rejects_new_raw_styling_even_inside_conditional_branches() {
    let source = r#"fn render() { if active { ui::Button::new("id", "Save"); } div().bg(color).rounded_md(); }
        #[cfg(test)] mod tests { fn fixture() { Label::new("Allowed fixture"); } }"#;
    let mut contract = Contract::default();
    contract.visit_file(&syn::parse_file(source).unwrap());
    assert_eq!(contract.problems.len(), 3);
}
