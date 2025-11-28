use anyhow::{bail, Context};
use chrono::Utc;
use mdbook_preprocessor::{
    book::{Book, BookItem},
    errors::Error,
    Preprocessor, PreprocessorContext,
};
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::fs;
use std::path::Path;

pub struct ContentLoader;

impl ContentLoader {
    pub fn new() -> ContentLoader {
        ContentLoader
    }
}

impl Default for ContentLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for ContentLoader {
    fn name(&self) -> &str {
        "content-loader"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // mdBook 0.5.1: Config has a typed book.src (PathBuf)
        let src = ctx.config.book.src.to_str().unwrap_or("src");
        let src_dir = ctx.root.join(src);
        let index_path = src_dir.join("content-collections.json");

        let payload: Value = match load_collections(&index_path) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("content-loader: {}", e);
                return Ok(book);
            }
        };

        let script = format!(
            r#"<script>window.CONTENT_COLLECTIONS = {};</script>"#,
            serde_json::to_string(&payload)?
        );

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = format!("{}\n{}", script, chapter.content);
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool, Error> {
        Ok(renderer == "html")
    }
}

fn load_collections(path: &Path) -> anyhow::Result<Value> {
    if !path.exists() {
        bail!("content-collections.json not found at {:?}", path);
    }

    let content = fs::read_to_string(path).context("Failed to read content-collections.json")?;
    let json_val: Value = serde_json::from_str(&content).context("Failed to parse JSON")?;

    let entries: Vec<Value> = json_val
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|a| a.to_vec())
        .unwrap_or_default();

    let published: Vec<_> = entries
        .into_iter()
        .filter(|e| !e.get("draft").and_then(|v| v.as_bool()).unwrap_or(false))
        .collect();

    let mut collections: Map<String, Value> = Map::new();
    let mut default_collection = vec![];

    for entry in &published {
        let coll = entry
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("posts")
            .to_string();
        if coll == "posts" {
            default_collection.push(entry.clone());
        } else {
            let entry_arr = collections
                .entry(coll)
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .expect("Failed to convert to array");
            entry_arr.push(entry.clone());
        }
    }

    if !default_collection.is_empty() {
        sort_by_date_desc(&mut default_collection);
        collections.insert("posts".to_string(), json!(default_collection));
    }

    for coll in collections.values_mut() {
        if let Value::Array(arr) = coll {
            sort_by_date_desc(arr);
        }
    }

    Ok(json!({
        "entries": published,
        "collections": collections,
        "generated_at": Utc::now().to_rfc3339(),
    }))
}

fn sort_by_date_desc(arr: &mut [Value]) {
    arr.sort_by_key(|e| {
        let date = e.get("date").and_then(|v| v.as_str()).unwrap_or("");
        Reverse(date.to_string())
    });
}
