use std::error::Error;
use std::process::Command;
fn main() -> Result<(), Box<dyn Error>> {
    let script = r#"nix-shell --run "code `pwd`""#;
    let output = Command::new("sh").arg("-c").arg(script).output()?;

    if output.status.success() {
        println!("nix-code command executed successfully");
    } else {
        eprintln!(
            "nix-code command failed with exit status: {:?}",
            output.status
        );
        println!(
            "nix-code stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        println!(
            "nix-code stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
