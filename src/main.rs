use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "git4", about = "A lightweight git clone in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new git4 repository
    Init,
    /// Compute object ID and optionally create a blob from a file
    HashObject {
        #[arg(short)]
        write: bool,
        file: String,
    },
    /// Provide content of repository objects
    CatFile {
        #[arg(short = 'p')]
        print: bool,
        object: String,
    },
    /// Create a tree object from the current index or workspace
    WriteTree,
    /// Create a new commit object
    CommitTree {
        tree: String,
        #[arg(short)]
        parent: Option<String>,
        #[arg(short)]
        message: String,
    },
    /// Add file contents to the index
    Add {
        files: Vec<String>,
    },
    /// Record changes to the repository
    Commit {
        #[arg(short)]
        message: String,
    },
    /// Show commit logs
    Log,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init()?,
        Commands::HashObject { write, file } => {
            let hash = hash_object(&file, write)?;
            println!("{}", hash);
        }
        Commands::CatFile { print, object } => {
            if print {
                let content = cat_file(&object)?;
                print!("{}", content);
            }
        }
        Commands::WriteTree => {
            let hash = write_tree(Path::new("."))?;
            println!("{}", hash);
        }
        Commands::CommitTree {
            tree,
            parent,
            message,
        } => {
            let hash = commit_tree(&tree, parent.as_deref(), &message)?;
            println!("{}", hash);
        }
        Commands::Add { files } => {
            add_files(files)?;
        }
        Commands::Commit { message } => {
            commit(&message)?;
        }
        Commands::Log => {
            log()?;
        }
    }

    Ok(())
}

/// Initialize the `.git4` directory structure
fn init() -> Result<()> {
    fs::create_dir(".git4")?;
    fs::create_dir(".git4/objects")?;
    fs::create_dir(".git4/refs")?;
    fs::create_dir(".git4/refs/heads")?;
    fs::write(".git4/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized empty git4 repository in .git4/");
    Ok(())
}

fn git4_dir() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let git4_path = current.join(".git4");
        if git4_path.exists() && git4_path.is_dir() {
            return Ok(git4_path);
        }
        if !current.pop() {
            return Err(anyhow!("Not a git4 repository (or any of the parent directories): .git4"));
        }
    }
}

/// Read object from `.git4/objects/xx/yyyy...`
fn read_object(hash: &str) -> Result<(String, Vec<u8>)> {
    let dir = git4_dir()?;
    let obj_path = dir.join("objects").join(&hash[0..2]).join(&hash[2..]);
    
    let compressed = fs::read(&obj_path).with_context(|| format!("Object {} not found", hash))?;
    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw)?;

    // format: `{type} {size}\0{content}`
    let nul_pos = raw.iter().position(|&b| b == 0).context("Invalid object format")?;
    let header = String::from_utf8(raw[0..nul_pos].to_vec())?;
    
    let parts: Vec<&str> = header.split(' ').collect();
    let obj_type = parts[0].to_string();
    // let size = parts[1].parse::<usize>()?;
    
    let content = raw[nul_pos + 1..].to_vec();
    Ok((obj_type, content))
}

fn write_object(obj_type: &str, content: &[u8]) -> Result<String> {
    let header = format!("{} {}\0", obj_type, content.len());
    let mut store = Vec::new();
    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(content);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let hash = hex::encode(hasher.finalize());

    let dir = git4_dir()?;
    let obj_dir = dir.join("objects").join(&hash[0..2]);
    if !obj_dir.exists() {
        fs::create_dir_all(&obj_dir)?;
    }
    let obj_path = obj_dir.join(&hash[2..]);
    
    if !obj_path.exists() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&store)?;
        let compressed = encoder.finish()?;
        fs::write(obj_path, compressed)?;
    }

    Ok(hash)
}

fn hash_object(file: &str, write: bool) -> Result<String> {
    let content = fs::read(file)?;
    if write {
        write_object("blob", &content)
    } else {
        let header = format!("blob {}\0", content.len());
        let mut store = Vec::new();
        store.extend_from_slice(header.as_bytes());
        store.extend_from_slice(&content);
        
        let mut hasher = Sha1::new();
        hasher.update(&store);
        Ok(hex::encode(hasher.finalize()))
    }
}

fn cat_file(hash: &str) -> Result<String> {
    let (obj_type, content) = read_object(hash)?;
    if obj_type == "blob" || obj_type == "commit" {
        Ok(String::from_utf8_lossy(&content).to_string())
    } else if obj_type == "tree" {
        // Simple tree parse logic to show its content
        let mut out = String::new();
        let mut i = 0;
        while i < content.len() {
            let space_pos = i + content[i..].iter().position(|&b| b == b' ').unwrap_or(0);
            let mode_str = String::from_utf8_lossy(&content[i..space_pos]);
            
            let nul_pos = space_pos + content[space_pos..].iter().position(|&b| b == 0).unwrap_or(0);
            let name_str = String::from_utf8_lossy(&content[space_pos+1..nul_pos]);
            
            let sha = hex::encode(&content[nul_pos+1..nul_pos+21]);
            
            out.push_str(&format!("{} {} {}\n", mode_str, sha, name_str));
            i = nul_pos + 21;
        }
        Ok(out)
    } else {
        Ok(format!("<{} object>", obj_type))
    }
}

fn write_tree(path: &Path) -> Result<String> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        
        if file_name == ".git4" || file_name == "target" || file_name.starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            let tree_hash = write_tree(&entry.path())?;
            entries.push((
                "40000".to_string(), // dir mode
                file_name,
                tree_hash,
            ));
        } else if file_type.is_file() {
            let meta = entry.metadata()?;
            let mode = if meta.permissions().mode() & 0o111 != 0 {
                "100755"
            } else {
                "100644"
            };
            let blob_hash = hash_object(entry.path().to_str().unwrap(), true)?;
            entries.push((mode.to_string(), file_name, blob_hash));
        }
    }

    // Sort by name
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut tree_content = Vec::new();
    for (mode, name, hash) in entries {
        tree_content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
        tree_content.extend_from_slice(&hex::decode(hash)?);
    }

    write_object("tree", &tree_content)
}

fn commit_tree(tree_hash: &str, parent_hash: Option<&str>, message: &str) -> Result<String> {
    let mut content = format!("tree {}\n", tree_hash);
    if let Some(parent) = parent_hash {
        content.push_str(&format!("parent {}\n", parent));
    }
    
    let author = "git4 User <git4@example.com>";
    let timestamp = Utc::now().timestamp();
    let tz = "+0000";
    
    content.push_str(&format!("author {} {} {}\n", author, timestamp, tz));
    content.push_str(&format!("committer {} {} {}\n", author, timestamp, tz));
    content.push_str("\n");
    content.push_str(message);
    content.push_str("\n");

    write_object("commit", content.as_bytes())
}

/// Helper function to retrieve the current HEAD commit hash
fn get_head() -> Result<Option<String>> {
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");
    if !head_path.exists() {
        return Ok(None);
    }
    
    let head_content = fs::read_to_string(head_path)?;
    let head_content = head_content.trim();
    
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.strip_prefix("ref: ").unwrap().trim();
        let full_ref_path = dir.join(ref_path);
        if full_ref_path.exists() {
            let hash = fs::read_to_string(full_ref_path)?;
            Ok(Some(hash.trim().to_string()))
        } else {
            Ok(None)
        }
    } else {
        Ok(Some(head_content.to_string()))
    }
}

fn update_head(hash: &str) -> Result<()> {
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");
    let head_content = fs::read_to_string(&head_path)?;
    let head_content = head_content.trim();
    
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.strip_prefix("ref: ").unwrap().trim();
        let full_ref_path = dir.join(ref_path);
        fs::write(full_ref_path, format!("{}\n", hash))?;
    } else {
        fs::write(head_path, format!("{}\n", hash))?;
    }
    Ok(())
}

fn add_files(files: Vec<String>) -> Result<()> {
    // For simplicity in git4, `add` will just hash the object into the store.
    // In a real git, it updates `.git/index`. Here we do a simplified index
    // in `.git4/index` formatted as simple lines.
    let dir = git4_dir()?;
    let index_path = dir.join("index");
    
    let mut index: BTreeMap<String, String> = BTreeMap::new();
    
    if index_path.exists() {
        let content = fs::read_to_string(&index_path)?;
        for line in content.lines() {
            if let Some((hash, path)) = line.split_once(' ') {
                index.insert(path.to_string(), hash.to_string());
            }
        }
    }
    
    for file in files {
        let path = Path::new(&file);
        if path.exists() && path.is_file() {
            let hash = hash_object(&file, true)?;
            println!("Added {} ({})", file, hash);
            index.insert(file.clone(), hash);
        } else {
            println!("Skipping {} (not found or not a regular file)", file);
        }
    }
    
    let mut new_index = String::new();
    for (path, hash) in index {
        new_index.push_str(&format!("{} {}\n", hash, path));
    }
    fs::write(index_path, new_index)?;
    
    Ok(())
}

fn commit(message: &str) -> Result<()> {
    // A simpler `commit` that uses `write-tree` on the whole workspace for now,
    // ignoring the actual complex index resolution.
    // Or we could build a tree from the `.git4/index` directly.
    // For `git4`, let's just write the whole tree (auto-commit behavior).
    println!("Building tree...");
    let tree_hash = write_tree(Path::new("."))?;
    println!("Tree hash: {}", tree_hash);
    
    let parent = get_head()?;
    let commit_hash = commit_tree(&tree_hash, parent.as_deref(), message)?;
    
    update_head(&commit_hash)?;
    println!("Committed: {}", commit_hash);
    
    Ok(())
}

fn log() -> Result<()> {
    let mut current = get_head()?;
    
    while let Some(hash) = current {
        let (obj_type, content) = read_object(&hash)?;
        if obj_type != "commit" {
            println!("HEAD is not a commit: {}", hash);
            break;
        }
        
        let content_str = String::from_utf8_lossy(&content);
        println!("commit {}", hash);
        
        let mut parent_hash = None;
        let mut is_msg = false;
        
        for line in content_str.lines() {
            if is_msg {
                println!("    {}", line);
            } else if line.is_empty() {
                is_msg = true;
                println!();
            } else if let Some(p) = line.strip_prefix("parent ") {
                parent_hash = Some(p.to_string());
            } else if line.starts_with("author ") || line.starts_with("committer ") {
                println!("{}", line);
            }
        }
        println!("\n");
        current = parent_hash;
    }
    
    Ok(())
}
