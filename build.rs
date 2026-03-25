use std::fs;
use std::path::Path;
use std::process::Command;

// This build script runs before the main Rust code is compiled.
// Its job is to compile Slang shader files into SPIR-V binaries.
fn main() {
    // Source shaders live in `shaders/`, and compiled output goes to `shaders/compiled/`.
    let shader_dir = Path::new("shader");
    let out_dir = shader_dir.join("compiled");
    fs::create_dir_all(&out_dir).unwrap();

    // Each entry is: (source .slang file, output SPV name, entry point name, stage).
    // A single .slang file typically contains both vertMain and fragMain,
    // so each source file appears twice — once per stage.
    let shaders = [
        ("main.slang", "main_vert.spv", "vertMain", "vertex"),
        ("main.slang", "main_frag.spv", "fragMain", "fragment"),
    ];

    for (src, dst, entry, stage) in &shaders {
        let src_path = shader_dir.join(src);
        let dst_path = out_dir.join(dst);

        // Tell Cargo to rerun this build script if a shader source file changes.
        println!("cargo:rerun-if-changed={}", src_path.display());

        // Run `slangc` to compile the Slang shader into SPIR-V.
        // -entry   : selects the entry point within the file (vertMain / fragMain)
        // -stage   : tells slangc which pipeline stage this compilation is for
        // -target  : emit SPIR-V binary
        // -o       : output path — must be a FILE, not a directory
        let output = Command::new("slangc")
            .arg(src_path.to_str().unwrap())
            .arg("-entry")
            .arg(entry)
            .arg("-stage")
            .arg(stage)
            .arg("-target")
            .arg("spirv")
            .arg("-o")
            .arg(dst_path.to_str().unwrap())
            .output()
            .expect("Failed to execute slangc. Is the Slang compiler installed?");

        // If compilation fails, stop the build and print the shader compiler error.
        if !output.status.success() {
            panic!(
                "Shader compilation failed for {} (entry: {}):\n{}",
                src,
                entry,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
