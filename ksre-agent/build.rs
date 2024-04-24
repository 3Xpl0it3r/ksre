use std::process::Command;

fn main() {
    let platform = Command::new("uname")
        .arg("-p")
        .output()
        .expect("execute command uname failed");
    let platform = String::from_utf8(platform.stdout)
        .unwrap()
        .trim()
        .to_string();
    match platform.as_str() {
        "x86_64" | "arm" => {}
        _ => {
            eprint!("platform [{}] is not supported", platform.as_str());
            std::process::exit(-1);
        }
    }
    println!("cargo:rustc-cfg=platform=\"{}\"", platform);
}
