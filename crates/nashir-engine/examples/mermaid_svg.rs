//! Renders a Mermaid file to SVG with Nashir's engine (the same output as
//! the app's "Export SVG"):
//!
//! ```sh
//! cargo run -p nashir-engine --example mermaid_svg -- diagram.mmd out.svg [--dark]
//! ```

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(input), Some(output)) = (args.first(), args.get(1)) else {
        eprintln!("usage: mermaid_svg <input.mmd> <output.svg> [--dark]");
        std::process::exit(2);
    };
    let dark = args.iter().any(|a| a == "--dark");
    let source = std::fs::read_to_string(input).expect("read the diagram");
    match nashir_engine::mermaid::render(&source) {
        Ok(scene) => {
            std::fs::write(output, nashir_engine::mermaid::svg::to_svg(&scene, dark)).expect("write the SVG");
            println!(
                "{}: {} × {} ({} shapes, {} labels)",
                scene.kind,
                scene.width,
                scene.height,
                scene.shapes.len(),
                scene.labels.len()
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
