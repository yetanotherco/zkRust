use log::{error, info};
use regex::Regex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, ErrorKind, Read, Seek, Write},
    path::{Path, PathBuf},
};

// Host
pub const IO_WRITE: &str = "zk_rust_io::write";
pub const IO_OUT: &str = "zk_rust_io::out();";
pub const HOST_INPUT: &str = "// INPUT //";
pub const HOST_OUTPUT: &str = "// OUTPUT //";

// I/O Markers
pub const IO_READ: &str = "zk_rust_io::read();";
pub const IO_COMMIT: &str = "zk_rust_io::commit";

pub const OUTPUT_FUNC: &str = r"pub fn output() {";
pub const INPUT_FUNC: &str = r"pub fn input() {";

pub fn prepend(file_path: &str, text_to_prepend: &str) -> io::Result<()> {
    // Open the file in read mode to read its existing content
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    // Read the existing content of the file
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Move the file cursor to the beginning of the file
    file.seek(io::SeekFrom::Start(0))?;

    // Write the text to prepend followed by the existing content back to the file
    file.write_all(text_to_prepend.as_bytes())?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn replace(file_path: &PathBuf, search_string: &str, replace_string: &str) -> io::Result<()> {
    // Read the contents of the file
    let mut contents = String::new();
    fs::File::open(file_path)?.read_to_string(&mut contents)?;

    // Replace all occurrences of the search string with the replace string
    let new_contents = contents.replace(search_string, replace_string);

    // Write the new contents back to the file
    let mut file = fs::File::create(file_path)?;
    file.write_all(new_contents.as_bytes())?;

    Ok(())
}

fn copy_dir_all(src: &impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn insert(target_file: &str, text: &str, search_string: &str) -> io::Result<()> {
    // Read the contents of the target file
    let mut target_contents = String::new();
    fs::File::open(target_file)?.read_to_string(&mut target_contents)?;

    // Find the position of the search string in the target file
    if let Some(pos) = target_contents.find(search_string) {
        // Split the target contents into two parts
        let (before, after) = target_contents.split_at(pos + search_string.len());

        // Combine the parts with the insert contents
        let new_contents = format!("{}\n{}\n{}", before, text, after);

        // Write the new contents back to the target file
        let mut file = fs::File::create(target_file)?;
        file.write_all(new_contents.as_bytes())?;
    } else {
        println!("Search string not found in target file.");
    }

    Ok(())
}

//Note: Works with a one off '{' not with '}'
pub fn extract_function_bodies(file_path: &PathBuf, functions: Vec<String>) -> io::Result<Vec<String>> {
    // Read the contents of the target file
    let mut code = String::new();
    fs::File::open(file_path)?.read_to_string(&mut code)?;

    let mut start_indices = vec![];
    let mut index = 0;

    // Find all start indices of the function signature
    for keyword in functions {
        if let Some(start_index) = code[index..].find(&keyword) {
            let absolute_index = index + start_index;
            start_indices.push(absolute_index);
            index = absolute_index + keyword.len();
        }
    }

    // Extract the code for each function
    let mut extracted_codes = vec![];
    for &start_index in &start_indices {
        if let Some(start_brace_index) = code[start_index..].find('{') {
            let start_brace_index = start_index + start_brace_index;
            let mut stack = vec!["{"];
            let mut end_index = start_brace_index;

            for (i, ch) in code[start_brace_index + 1..].chars().enumerate() {
                if handle_stack(ch, &mut stack) {
                    end_index = start_brace_index + 1 + i;
                    break;
                }
            }

            let extracted_code = &code[start_brace_index + 1..end_index].trim();
            extracted_codes.push(extracted_code.to_string());
        }
    }

    Ok(extracted_codes)
}

// Function that handles the stack and status when parsing the file to extract_function_bodies
fn handle_stack(ch: char, stack: &mut Vec<&str>) -> bool {
    match stack.last() {
        Some(&"{") => return handle_char(ch, stack),
        Some(&"/") => match ch {
            '/' => {
                stack.pop();
                stack.push("//comment");
            }
            '*' => {
                stack.pop();
                stack.push("/*comment*\\");
            }
            _ => {
                stack.pop();
                handle_char(ch, stack);
            }
        },
        Some(&"//comment") => match ch {
            '\n' => {
                stack.pop();
            }
            _ => {}
        },
        Some(&"/*comment*\\") => match ch {
            '*' => {
                stack.push("*");
            }
            _ => {}
        },
        Some(&"*") => {
            match ch {
                '/' => {
                    stack.pop(); //pop("*")
                    stack.pop(); //pop("/*comment*\\")
                }
                _ => {
                    stack.pop(); //pop("*"), back to "/*comment*\\"
                }
            }
        }
        Some(&"\"string\"") => match ch {
            '\"' => {
                stack.pop();
            }
            _ => {}
        },
        Some(&"\'c\'") => match ch {
            '\'' => {
                stack.pop();
            }
            _ => {}
        },
        _ => {}
    }
    false
}
// Function to handle characters when in normal status of the stack
fn handle_char(ch: char, stack: &mut Vec<&str>) -> bool {
    match ch {
        '/' => {
            stack.push("/");
        }
        '{' => stack.push("{"),
        '}' => {
            stack.pop();
            if stack.is_empty() {
                return true;
            }
        }
        '\"' => {
            stack.push("\"string\"");
        }
        '\'' => {
            stack.push("\'c\'");
        }
        _ => {}
    }
    false
}

fn copy_dependencies(toml_path: &PathBuf, guest_toml_path: &PathBuf) -> io::Result<()> {
    let mut toml = std::fs::File::open(toml_path)?;
    let mut content = String::new();
    toml.read_to_string(&mut content)?;

    match content.find("[dependencies]") {
        Some(start_index) => {
            // Get all text after the search string
            let dependencies = &content[start_index + "[dependencies]".len()..];
            // Open the output file in append mode
            let mut guest_toml = OpenOptions::new()
                .create(true)
                .append(true)
                .open(guest_toml_path)
                .unwrap();

            // Write the text after the search string to the output file
            guest_toml.write_all(dependencies.as_bytes())
        }
        None => Err(io::Error::other(
            "Failed to find `[dependencies]` in project Cargo.toml",
        )),
    }
}

pub fn prepare_workspace(
    guest_path: &PathBuf,
    workspace_guest_dir: &PathBuf,
    program_toml_dir: &PathBuf,
    workspace_host_dir: &PathBuf,
    host_toml_dir: &PathBuf,
    base_host_toml_dir: &PathBuf,
    base_guest_toml_dir: &PathBuf,
) -> io::Result<()> {
    // Create proof_data directory
    //TODO: const
    let proof_data_dir = PathBuf::from("./proof_data");
    if !proof_data_dir.exists() {
        std::fs::create_dir_all(proof_data_dir)
            .unwrap_or(info!("saving generated proofs to `proof_data`"));
    }
    let workspace_guest_src_dir = workspace_guest_dir.join("src");
    let workspace_host_src_dir = workspace_host_dir.join("src");
    if let Err(e) = fs::remove_dir_all(&workspace_guest_src_dir) {
        if e.kind() != ErrorKind::NotFound {
            return Err(e);
        }
    }
    if let Err(e) = fs::remove_dir_all(&workspace_host_src_dir) {
        if e.kind() != ErrorKind::NotFound {
            return Err(e);
        }
    }
    // Copy src/ directory
    let src_dir_path = guest_path.join("src");
    copy_dir_all(&src_dir_path, workspace_guest_src_dir)?;
    copy_dir_all(&src_dir_path, workspace_host_src_dir)?;

    // Copy lib/ if present
    let lib_dir_path = guest_path.join("lib");
    if Path::new(&lib_dir_path).exists() {
        let workspace_guest_lib_dir = workspace_guest_dir.join("lib");
        let workspace_host_lib_dir = workspace_host_dir.join("lib");
        copy_dir_all(&lib_dir_path, workspace_guest_lib_dir)?;
        copy_dir_all(&lib_dir_path, workspace_host_lib_dir)?;
    }

    // Copy Cargo.toml for zkVM
    fs::copy(base_guest_toml_dir, &program_toml_dir)?;
    println!("{:?} {:?}", base_guest_toml_dir, program_toml_dir);
    fs::copy(base_host_toml_dir, &host_toml_dir)?;

    // Select dependencies from the
    let toml_path = guest_path.join("Cargo.toml");
    copy_dependencies(&toml_path, &program_toml_dir)?;
    copy_dependencies(&toml_path, &host_toml_dir)?;

    println!("something");
    Ok(())
}

//TODO: refactor this to eliminate the clone at each step.
pub fn get_imports(filename: &PathBuf) -> io::Result<String> {
    // Open the file
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let mut imports = String::new();

    // Read the file line by line
    while let Some(line) = lines.next() {
        let mut line = line?;
        // Check if the line starts with "use "
        if line.trim_start().starts_with("use ")
            || line.trim_start().starts_with("pub mod ")
            || line.trim_start().starts_with("mod ")
        {
            line.push('\n');
            imports.push_str(&line.clone());
            // check if line does not contains a use declarator and a ';'
            // if not continue reading till one is found this covers the case where import statements cover multiple lines
            if !line.contains(';') {
                // Iterate and continue adding lines to the import while line does not contain a ';' break if it does
                for line in lines.by_ref() {
                    let mut line = line?;
                    line.push('\n');
                    imports.push_str(&line.clone());
                    if line.contains(';') {
                        break;
                    }
                }
            }
        }
    }

    Ok(imports)
}

pub fn extract_regex(file_path: &PathBuf, regex: &str) -> io::Result<Vec<String>> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut values = Vec::new();
    let regex = Regex::new(regex).unwrap();

    for line in reader.lines() {
        let line = line?;
        for cap in regex.captures_iter(&line) {
            if let Some(matched) = cap.get(1) {
                values.push(matched.as_str().to_string());
            }
        }
    }

    Ok(values)
}

//Change to remove regex and remove the marker
pub fn remove_lines(file_path: &PathBuf, target: &str) -> io::Result<()> {
    // Read the file line by line
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    // Collect lines that do not contain the target string
    let lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.contains(target))
        .collect();

    // Write the filtered lines back to the file
    let mut file = fs::File::create(file_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn validate_directory_structure(root: &str) -> bool {
    let root = Path::new(root);
    // Check if Cargo.toml exists in the root directory
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.exists() {
        error!("Cargo.toml not found.");
        return false;
    }

    // Check if src/ and lib/ directories exist
    let src_dir = root.join("src");

    if !src_dir.exists() {
        error!("src/ directory not found in root");
        return false;
    }

    // Check if src/ contains main.rs file
    let main_rs = src_dir.join("main.rs");
    if !main_rs.exists() {
        error!("main.rs not found in src/ directory in root");
        return false;
    }

    true
}

pub fn prepare_guest(
    imports: &str,
    main_func_code: &str,
    program_header: &str,
    io_read_header: &str,
    io_commit_header: &str,
    guest_main_file_path: &PathBuf,
) -> io::Result<()> {
    let mut guest_program = program_header.to_string();
    guest_program.push_str(imports);
    guest_program.push_str("pub fn main() {\n");
    guest_program.push_str(main_func_code);
    guest_program.push_str("\n}");

    // Replace zkRust::read()
    let guest_program = guest_program.replace(IO_READ, io_read_header);

    // Replace zkRust::commit()
    let guest_program = guest_program.replace(IO_COMMIT, io_commit_header);

    // Write to guest
    let mut file = fs::File::create(guest_main_file_path)?;
    file.write_all(guest_program.as_bytes())?;
    Ok(())
}
