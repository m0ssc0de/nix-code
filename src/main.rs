use clap::Parser;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
mod files;
mod home;
mod tags;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to operate on
    ///
    /// This can be a directory or a file that will be opened in vscode. If it's a directory, the program may search for a ".nix" file in it to set up the development environment. If no ".nix" file is found, the program will search our nix file index using a "tag" derived from its git remote. For example, if the git remote is "git@github.com:rust-lang/cargo.git", the tag will be "github.com/rust-lang/cargo".
    #[arg(default_value = ".", value_name = "PATH")]
    path: PathBuf,

    /// Sets a custom nix file
    ///
    /// This should be the path to a nix file, like "./shell.nix". Its priority is higher than the nix file in path.
    #[arg(short, long, value_name = "NIX FILE")]
    file: Option<PathBuf>,

    /// Sets a custom nix file tag
    ///
    /// This tag is used to identify the nix file in a more user-friendly way. It's used to search for a nix file in our nix file index. "NIX FILE" and "NIX FILE TAG" cannot be set at the same time. Its priority is higher than the nix file in path.
    ///
    /// You can directly set the tag to search in the index for projects of the same type that already have nix files, such as "github.com/NixOS/nix".
    /// Or set the tag like "rust" to use a predefined nix file for a specific type of project.
    #[arg(short, long, value_name = "NIX FILE TAG")]
    tag: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    home::init()?;
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
    let cli = Cli::parse();

    let work_dir = cli.path;
    let nix_file = cli.file;
    let nix_tag = match cli.tag {
        Some(nix_tag) => Some(nix_tag),
        None => tags::get_tag(work_dir.as_path()),
    };

    let nix_file_by_tag = match nix_tag {
        Some(nix_tag) => files::find_nix_file(Path::new(
            format!(
                "{}/{}/{}/{}",
                home_dir,
                home::NIX_CODE_DIR,
                home::INDEX_DIR,
                nix_tag
            )
            .as_str(),
        )),
        None => None,
    };
    let nix_file_in_work_dir = files::find_nix_file(&work_dir);
    let nix_file = match nix_file {
        Some(nix_file) => Path::new(&nix_file).to_path_buf(),
        None => match nix_file_by_tag {
            Some(nix_file) => nix_file,
            None => match nix_file_in_work_dir {
                Some(nix_file) => nix_file,
                None => {
                    eprintln!("No nix file found in the work directory");
                    return Ok(());
                }
            },
        },
    };

    let script = format!(
        r#"nix-shell {} --run "code {}""#,
        nix_file.display(),
        work_dir.display()
    );
    println!("Executing: {}", script);
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdout(Stdio::inherit()) // Pipe stdout to current stdout
        .stderr(Stdio::inherit()) // Pipe stderr to current stderr
        .output()?;

    if output.status.success() {
        println!("nix-code command executed successfully");
    } else {
        eprintln!(
            "nix-code command failed with exit status: {:?}",
            output.status
        );
    }

    Ok(())
}
