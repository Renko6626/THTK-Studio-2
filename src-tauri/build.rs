use std::{env, fs, path::Path};

fn collect(root: &Path, dir: &Path, entries: &mut Vec<(String, std::path::PathBuf)>) {
    for entry in fs::read_dir(dir).expect("read docs/agent") {
        let path = entry.expect("read agent document").path();
        if path.is_dir() {
            collect(root, &path, entries);
        } else if path.is_file() {
            entries.push((
                path.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                path,
            ));
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/agent");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut entries = Vec::new();
    collect(&root, &root, &mut entries);
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut generated = String::from("const AGENT_REFERENCES: &[(&str, &[u8])] = &[\n");
    for (relative, path) in entries {
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
