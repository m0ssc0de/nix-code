use chrono;
use flate2::read::GzDecoder;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tar::Archive;

const LATEST_INDEX_URL: &str =
    "https://github.com/m0ssc0de/nix-code-index/archive/refs/heads/main.tar.gz";
pub const NIX_CODE_DIR: &str = ".nix-code";
pub const INDEX_DIR: &str = "nix-code-index-main";
const TMP_TAR: &str = "/tmp/repo2.tar.gz";

pub fn init() -> std::io::Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("which nix")
        .output()
        .expect("failed to execute process");
    if !output.status.success() {
        eprintln!("Nix is not installed, please install nix by running:\n sh <(curl -L https://nixos.org/nix/install)");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Nix is not installed",
        ));
    }

    let home_dir = env::var("HOME").expect("HOME directory not set");
    let nix_code_dir = Path::new(&home_dir).join(NIX_CODE_DIR);
    fs::create_dir_all(&nix_code_dir)?;

    // if the index directory does exist, download and extract it
    if nix_code_dir.join(INDEX_DIR).exists() {
        // read timestamp file, if it's older than 1 day, download the index
        let timestamp_file = nix_code_dir.join(format!("{}-timestamp", NIX_CODE_DIR));
        if timestamp_file.exists() {
            let timestamp = fs::read_to_string(&timestamp_file)?;
            let timestamp = timestamp.parse::<i64>().unwrap();
            let now = chrono::Utc::now().timestamp();
            if now - timestamp < 86400 {
                return Ok(());
            }
        }
    }

    println!("Downloading index");
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap();
    let response = client
        .get(LATEST_INDEX_URL)
        .send()
        .expect("Failed to download tarball");
    let tarball_path = Path::new(TMP_TAR);
    let file = File::create(&tarball_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&response.bytes().unwrap())?;
    writer.flush()?;
    // write current timestamp to file
    let mut file = File::create(nix_code_dir.join(format!("{}-timestamp", NIX_CODE_DIR)))?;
    file.write_all(format!("{}", chrono::Utc::now().timestamp()).as_bytes())?;
    file.flush()?;

    let tarball = File::open(tarball_path)?;
    if tarball.metadata()?.len() == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Tarball is empty",
        ));
    }

    let tar_gz = File::open(tarball_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(nix_code_dir)?;

    Ok(())
}
