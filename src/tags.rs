use std::path::Path;

use url::Url;

pub fn get_tag(work_dir: &Path) -> Option<String> {
    let git_remote = gix_config::File::from_git_dir(work_dir.join(".git"))
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
                Some(format!(
                    "{}/{}/{}",
                    host,
                    username,
                    repo_name.trim_end_matches(".git")
                ))
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
                    Some(format!("{}/{}/{}", host, username, repo_name))
                } else {
                    None
                }
            }
        };
        return github_remote_name;
    } else {
        return None;
    };
}
