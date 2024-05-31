use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let project_dir = Path::new(".");
    let script_file = "collect_todos.rs";
    let output_dir = Path::new("./scripts");
    let output_file = output_dir.join("todos.txt");

    // Collect all TODO comments
    let todos = collect_todos(project_dir, script_file)?;

    // Write TODO comments to the output file
    write_todos_to_file(&output_file, &todos)?;

    Ok(())
}

fn collect_todos(dir: &Path, script_file: &str) -> io::Result<Vec<(String, String)>> {
    let mut todos = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && !is_cargo_dependency(&path) {
            // Recursively search in subdirectories
            let subdir_todos = collect_todos(&path, script_file)?;
            todos.extend(subdir_todos);
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            // Skip the script file itself
            if path.file_name().map_or(false, |name| name == script_file) {
                continue;
            }
            // Extract TODO comments from the file
            let file_todos = extract_todos(&path)?;
            todos.extend(file_todos);
        }
    }

    Ok(todos)
}

fn is_cargo_dependency(path: &Path) -> bool {
    path.starts_with("target")
}

fn extract_todos(path: &Path) -> io::Result<Vec<(String, String)>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut todos = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if let Some(pos) = line.find("TODO") {
            let todo = format!("{}:{}: {}", path.display(), index + 1, line[pos..].trim());
            todos.push((todo, path.to_string_lossy().to_string()));
        }
    }

    Ok(todos)
}

fn write_todos_to_file(output_file: &Path, todos: &[(String, String)]) -> io::Result<()> {    
    let mut file = File::create(output_file)?;
    writeln!(file, "TODOs in project:")?;

    for (todo, path) in todos {
        writeln!(file, "{} {}", todo, path)?;
    }

    Ok(())
}
