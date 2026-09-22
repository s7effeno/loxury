use crate::vm::Vm;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_lox(source: &str) -> String {
    let mut vm = Vm::with_outputs(Vec::new(), Vec::new());
    let ok = vm.run(source).is_ok();
    let output = String::from_utf8(std::mem::take(&mut vm.out)).unwrap_or_default();
    let error = String::from_utf8(std::mem::take(&mut vm.err)).unwrap_or_default();
    serde_json::json!({
        "ok": ok,
        "output": output.lines().collect::<Vec<_>>(),
        "error": (!error.is_empty()).then(|| error.trim_end().to_owned()),
    })
    .to_string()
}
