use std::{env, fs, path::Path};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/agent");
    println!("cargo:rerun-if-changed={}", root.display());
    // Maintenance notes and source examples stay in the repository.
    let mut generated = String::from("const AGENT_REFERENCES: &[(&str, &[u8])] = &[\n");
    for relative in [
        "README.md",
        "msg.md",
        "msg-instructions.md",
        "ending.md",
        "ending-instructions.md",
        "std.md",
        "std-instructions.md",
    ] {
        let path = root.join(relative);
        generated.push_str(&format!(
            "({relative:?}, include_bytes!({:?})),\n",
            path.canonicalize().unwrap()
        ));
    }
    generated.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("agent_references.rs"),
        generated,
    )
    .expect("embed agent documents");
    tauri_build::build()
}
