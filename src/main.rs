use git2::Repository;
use std::env;
use std::error::Error;

fn walkGitLog() -> Result<Vec<String>, Box<dyn Error>>{
    let repoPath = env::current_dir()?;
    let repo = match Repository::open(repoPath) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open repo: {}", e),
    };

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    Ok(revwalk
        .map(|curRev| {
            let oid = curRev.expect("error getting current revision");
            match repo.find_commit(oid) {
                Ok(commit) => commit.message()
                                    .expect("error getting current commit message")
                                    .to_string(),
                Err(e) => {
                    panic!("Error walking the revisions: {}", e)
                }
            }
        })
        .collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    let commitMessages = match walkGitLog() {
        Ok(commitMessage) => commitMessage,
        Err(e) => panic!("error getting commit message: {}", e)
    };
    for message in commitMessages {
        println!("{}", message);
    }
    let ollama = Ollama::default();
    Ok(())
}
