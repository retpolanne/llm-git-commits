use chromadb::v1::client::ChromaClient;
use chromadb::v1::collection::{ChromaCollection, CollectionEntries};
use git2::Repository;
use ollama_rs::Ollama;
use std::env;
use std::error::Error;

fn walk_git_log() -> Result<Vec<String>, Box<dyn Error>> {
    let repo_path = env::current_dir()?;
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open repo: {}", e),
    };

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    Ok(revwalk
        .map(|cur_rev| {
            let oid = cur_rev.expect("error getting current revision");
            match repo.find_commit(oid) {
                Ok(commit) => commit
                    .message()
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
    let chroma_client: ChromaClient = ChromaClient::new(Default::default());
    let collection: ChromaCollection =
        chroma_client.get_or_create_collection("commit_collection", None)?;
    let ollama = Ollama::default();

    for commit_enum in walk_git_log()
        .expect("error getting git log")
        .into_iter()
        .enumerate()
    {
        let embeddings = ollama
            .generate_embeddings(
                "mxbai-embed-large".to_string(),
                commit_enum.1.to_string(),
                None,
            )
            .await
            .expect("error generating embeddings")
            .embeddings
            .into_iter()
            .map(|embedding| embedding as f32)
            .collect();
        collection.add(
            CollectionEntries {
                ids: vec![&commit_enum.0.to_string()],
                documents: Some(vec![&commit_enum.1.to_string()]),
                embeddings: Some(vec![embeddings]),
                metadatas: Some(vec![]),
            },
            None,
        );
    }
    Ok(())
}
