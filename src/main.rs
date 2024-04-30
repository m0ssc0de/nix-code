use std::env;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use url::Url;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir()?;

    let (nix_file, work_dir) = match args.len() {
        1 => (None, current_dir),
        2 => {
            let arg1 = &args[1];
            let path = Path::new(arg1);
            if path.is_file() && path.extension().map_or(false, |ext| ext == "nix") {
                (Some(arg1), current_dir)
            } else {
                (None, path.to_path_buf())
            }
        }
        3 => (Some(&args[1]), Path::new(&args[2]).to_path_buf()),
        _ => return Err("Invalid number of arguments. Expected 0, 1 or 2.".into()),
    };
    if args.len() > 3 {
        return Err("Invalid number of arguments. Expected 0, 1 or 2.".into());
    }

    let git_remote = gix_config::File::from_git_dir(work_dir.clone().join(".git"))
        .ok()
        .and_then(|g| {
            g.strings_by_key("remote.origin.url")
                .and_then(|vec| vec.first().map(|s| s.to_string()))
        });
    println!("Git Remote: {:?}", git_remote);
    if let Some(remote_url) = git_remote {
        let github_remote_name = match Url::parse(&remote_url) {
            Ok(url) => {
                let host = url.host_str().unwrap().to_string();
                let mut path_segments = url.path_segments().unwrap();
                let repo_name = path_segments.next_back().unwrap();
                let username = path_segments.next_back().unwrap();
                format!(
                    "url {}/{}/{}",
                    host,
                    username,
                    repo_name.trim_end_matches(".git")
                )
            }
            Err(_) => {
                if remote_url.starts_with("git@") {
                    let host = remote_url
                        .split(':')
                        .nth(0)
                        .unwrap_or("")
                        .trim_start_matches("git@");
                    let path = remote_url.split(':').nth(1).unwrap_or("");
                    let mut path_segments = path.split('/');
                    let username = path_segments.next().unwrap_or("");
                    let repo_name = path_segments.next().unwrap_or("").trim_end_matches(".git");
                    format!("git {}/{}/{}", host, username, repo_name)
                } else {
                    String::new()
                }
            }
        };
        println!("GitHub Remote Name: {}", github_remote_name);
    }

    let nix_file = match nix_file {
        Some(nix_file) => Path::new(nix_file).to_path_buf(),
        None => {
            let mut nix_files = work_dir
                .read_dir()?
                .filter_map(|entry| {
                    entry.ok().and_then(|entry| {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "nix") {
                            Some(path)
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();

            // Sort nix_files so that shell.nix comes first, then default.nix, then others
            nix_files.sort_by(|a, b| {
                let a_str = a.file_name().unwrap().to_str().unwrap();
                let b_str = b.file_name().unwrap().to_str().unwrap();

                if a_str == "shell.nix" {
                    std::cmp::Ordering::Less
                } else if b_str == "shell.nix" {
                    std::cmp::Ordering::Greater
                } else if a_str == "default.nix" {
                    std::cmp::Ordering::Less
                } else if b_str == "default.nix" {
                    std::cmp::Ordering::Greater
                } else {
                    a_str.cmp(b_str)
                }
            });

            // Choose the first nix file, or return an error if there are none
            nix_files
                .first()
                .ok_or("No .nix files found in the directory {}")?
                .to_path_buf()
        }
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
