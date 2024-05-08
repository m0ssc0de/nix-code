use std::{path::PathBuf, process::Command};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "SSH HOST")]
    ssh_host: String,

    #[arg(value_name = "PATH TO OPEN with VSCode")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let ssh_host = cli.ssh_host;
    let path = cli.path;
    let path = path.to_str().unwrap();
    // code --remote ssh-remote+ssh://pi5 /tmp -n
    let script = format!(
        r#"code {} {} -n"#,
        format!("--remote ssh-remote+ssh://{}", ssh_host),
        path
    );
    println!("Executing: {}", script);
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to execute process");
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}
