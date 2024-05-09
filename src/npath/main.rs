use clap::{arg, Parser, Subcommand};
use serde_yaml;
use ssh2::Session;
use ssh2_config::{ParseRule, SshConfig};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{BufReader, Read};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Subcommand)]
enum Commands {
    Scan {
        /// Path to scan
        #[arg(default_value = "~", value_name = "SCAN PATH")]
        path: PathBuf,

        /// Sets a custom nix file
        #[arg(default_value = "2", short, long, value_name = "SCAN DEPTH")]
        depth: u32,

        /// Sets a custom nix file tag
        #[arg(short, long, value_name = "SCAN HOST")]
        ssh_host: String,
    },
    List,
    Load,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan {
            path,
            depth,
            ssh_host,
        } => {
            scan(path, depth, ssh_host);
        }
        Commands::List => {
            list();
        }
        Commands::Load => {
            load();
        }
    }
}

fn load() {
    let completion_yaml = ".nix-code/completion.yaml";
    let completion_script = ".nix-code/completion";
    let zshrc = ".zshrc";
    let home_path = dirs::home_dir().unwrap();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let completion_yaml = home_path.join(completion_yaml);
    if completion_yaml.exists() {
        let file = File::open(completion_yaml.clone()).unwrap();
        let loaded_map: HashMap<String, Vec<String>> = serde_yaml::from_reader(file).unwrap();
        map.extend(loaded_map);
    }
    let mut results = Vec::new();
    let mut keys = Vec::new();
    for (subcmd, values) in &map {
        let result = render_template(subcmd, values);
        keys.push(subcmd.clone());
        results.push(result);
    }
    let s = TEMPLATE
        .replace("{HOSTS_LEVEL}", &2.to_string())
        .replace("{HOSTS}", &results.join("\n        "))
        .replace("{PATHS}", &keys.join("\" \""));

    let script_path = home_path.join(completion_script);
    let mut file = File::create(script_path).unwrap();
    file.write_all(s.as_bytes()).unwrap();

    let zshrc_path = home_path.join(zshrc);
    let mut file = File::open(&zshrc_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let append_string = format!("\nsource ~/{}\n", completion_script);
    if !contents.contains(&append_string) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&zshrc_path)
            .unwrap();
        file.write_all(append_string.as_bytes()).unwrap();
    }
    println!("please run `source ~/.zshrc` to enable completion");
}

fn list() {
    let home_path = dirs::home_dir().unwrap();
    let ssh_config = ".ssh/config";
    let mut reader = BufReader::new(
        File::open(home_path.join(ssh_config)).expect("Could not open configuration file"),
    );
    let config = SshConfig::default()
        .parse(&mut reader, ParseRule::STRICT)
        .expect("Failed to parse configuration");
    // print config.host
    // iterate over hosts
    for host in config.get_hosts() {
        println!("Host: {:?}", host.pattern.first().unwrap().pattern);
    }
}

fn scan(path: PathBuf, depth: u32, ssh_host: String) {
    let target_path = path;
    let depth = depth;
    let ssh_host = &ssh_host;
    let home_path = dirs::home_dir().unwrap();
    let ssh_config = ".ssh/config";
    let completion_yaml = ".nix-code/completion.yaml";
    let mut reader = BufReader::new(
        File::open(home_path.join(ssh_config)).expect("Could not open configuration file"),
    );
    let config = SshConfig::default()
        .parse(&mut reader, ParseRule::STRICT)
        .expect("Failed to parse configuration");

    // Query attributes for a certain host
    let params = config.query(ssh_host);
    println!("Host: {:?}", params);

    let host = match params.host_name.as_deref() {
        Some(h) => h,
        None => ssh_host,
    };
    let port = match params.port {
        None => 22,
        Some(p) => p,
    };
    let host = match host.contains(':') {
        true => host.to_string(),
        false => format!("{}:{}", host, port),
    };

    let socket_addresses: Vec<SocketAddr> = match host.to_socket_addrs() {
        Ok(s) => s.collect(),
        Err(err) => {
            panic!("Could not parse host: {}", err);
        }
    };

    let mut tcp: Option<TcpStream> = None;
    // Try addresses
    for socket_addr in socket_addresses.iter() {
        match TcpStream::connect_timeout(
            socket_addr,
            params.connect_timeout.unwrap_or(Duration::from_secs(30)),
        ) {
            Ok(stream) => {
                println!("Established connection with {}", socket_addr);
                tcp = Some(stream);
                break;
            }
            Err(_) => continue,
        }
    }

    // If stream is None, return connection timeout
    let stream: TcpStream = match tcp {
        Some(t) => t,
        None => {
            panic!("No suitable socket address found; connection timeout");
        }
    };
    let user_name = params.user.unwrap();
    let s = connect_ssh_with_private_key(
        stream,
        &user_name,
        params.identity_file.unwrap().first().unwrap(),
    )
    .unwrap();

    // exec command
    let mut channel = s.channel_session().unwrap();
    // channel.exec(&format!("id -u {}", user_name)).unwrap();
    channel
        .exec(&format!(
            "find {} -maxdepth {} -type d",
            target_path.display(),
            depth
        ))
        .unwrap();
    let mut user_id = String::new();
    channel.read_to_string(&mut user_id).unwrap();

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let completion_yaml = home_path.join(completion_yaml);
    if completion_yaml.exists() {
        let file = File::open(completion_yaml.clone()).unwrap();
        let loaded_map: HashMap<String, Vec<String>> = serde_yaml::from_reader(file).unwrap();
        map.extend(loaded_map);
    }

    map.insert(
        ssh_host.to_string(),
        user_id.trim().split('\n').map(|s| s.to_string()).collect(),
    );
    let yaml_string = serde_yaml::to_string(&map).unwrap();
    let mut file = File::create(completion_yaml).unwrap();
    file.write_all(yaml_string.as_bytes()).unwrap();
}
fn render_template(subcmd: &str, values: &[String]) -> String {
    let value = values.join(" ");
    format!("[\"{}\"]=\"{}\"", subcmd, value)
}

fn connect_ssh_with_private_key(
    tcp: TcpStream,
    username: &str,
    private_key_path: &PathBuf,
) -> Result<Session, Box<dyn std::error::Error>> {
    // Create a new SSH session
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate with the private key
    sess.userauth_pubkey_file(username, None, private_key_path, None)?;

    // Check if session is authenticated
    if sess.authenticated() {
        Ok(sess)
    } else {
        Err("Authentication failed".into())
    }
}

const TEMPLATE: &str = r#"# Define the function for completion
_ssh_path_() {
    local -a options

    # Define an associative array for subcommands and their corresponding directory strings
    typeset -A subcmds
    subcmds=(
        {HOSTS}
    )

    local last_2=${words[-2]}
    local last_3=${words[-3]}

    if [[ $last_2 == "-s" || $last_2 == "--ssh-host" ]]; then
        options=("{PATHS}")
    elif [[ $last_3 == "-s" || $last_3 == "--ssh-host" ]]; then
        options=(${=subcmds[$last_2]})
    fi

    _describe 'values' options
}

# Associate the function with your command using compdef
compdef _ssh_path_ scd
compdef _ssh_path_ ncd"#;
