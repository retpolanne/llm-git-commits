# llm-git-commits

Use LLMs to summarize git commit messages

## Installation

Install the dependencies (macOS):

```sh
brew bundle
brew services start ollama
ollama pull mxbai-embed-large
```

Running chromadb (docker):

``` sh
docker run --rm -d --name chromadb -p 8000:8000 chromadb/chroma:latest
```


