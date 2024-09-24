use git2::Repository;
use std::env;
use std::error::Error;
use chromadb::v1::client::ChromaClient;
use chromadb::v1::collection::{ChromaCollection, CollectionEntries};
use ollama_rs::Ollama;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chromaClient: ChromaClient = ChromaClient::new(Default::default());
    let collection: ChromaCollection = chromaClient.get_or_create_collection(
        "commit_collection", None
    )?;
    let ollama = Ollama::default();

    walkGitLog()
        .expect("error getting git log")
        .into_iter()
        .enumerate()
        .map(|commitEnum| async {
            let embeddings = ollama.generate_embeddings(
                "mxbai-embed-large".to_string(), commitEnum.1.to_string(), None)
                                   .await
                                   .expect("error generating embeddings")
                                   .embeddings
                                   .into_iter()
                                   .map(|embedding| embedding as f32)
                                   .collect();
            collection.add(CollectionEntries{
                ids: vec![commitEnum.0.to_string().as_str()],
                documents: Some(vec![commitEnum.1.to_string().as_str()]),
                embeddings: Some(vec![embeddings]),
                metadatas: Some(vec![])
            }, None);
        });
    Ok(())
}
