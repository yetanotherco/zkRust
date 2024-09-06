use regex::Regex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, ErrorKind, Read, Seek, Write},
    path::Path,
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

pub fn prepend_to_file(file_path: &str, text_to_prepend: &str) -> io::Result<()> {
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

pub fn replace(file_path: &str, search_string: &str, replace_string: &str) -> io::Result<()> {
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
pub fn extract_function_bodies(file_path: &str, functions: Vec<String>) -> io::Result<Vec<String>> {
    // Read the contents of the target file
    let mut code = String::new();
    fs::File::open(file_path)?.read_to_string(&mut code)?;

    let mut start_indices = vec![];
    let mut index = 0;

    // Find all start indices of the function signature
    for keyword in functions {
        let start_index = code[index..].find(&keyword).unwrap();
        let absolute_index = index + start_index;
        start_indices.push(absolute_index);
        index = absolute_index + keyword.len();
    }

    // Extract the code for each function
    let mut extracted_codes = vec![];
    for &start_index in &start_indices {
        if let Some(start_brace_index) = code[start_index..].find('{') {
            let start_brace_index = start_index + start_brace_index;
            let mut stack = vec!['{'];
            let mut end_index = start_brace_index;

            for (i, ch) in code[start_brace_index + 1..].chars().enumerate() {
                match ch {
                    '{' => stack.push('{'),
                    '}' => {
                        stack.pop();
                        if stack.is_empty() {
                            end_index = start_brace_index + 1 + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let extracted_code = &code[start_brace_index + 1..end_index].trim();
            extracted_codes.push(extracted_code.to_string());
        }
    }

    Ok(extracted_codes)
}

//TODO: make this smarter -> end if next '[' found?
fn copy_dependencies(toml_path: &str, guest_toml_path: &str) {
    let mut toml = std::fs::File::open(toml_path).unwrap();
    let mut content = String::new();
    toml.read_to_string(&mut content).unwrap();

    if let Some(start_index) = content.find("[dependencies]") {
        // Get all text after the search string
        let dependencies = &content[start_index + "[dependencies]".len()..];
        // Open the output file in append mode
        let mut guest_toml = OpenOptions::new()
            .create(true)
            .append(true)
            .open(guest_toml_path)
            .unwrap();

        // Write the text after the search string to the output file
        guest_toml.write_all(dependencies.as_bytes()).unwrap();
    } else {
        println!("Failed to copy dependencies in Guest Toml file, plese check");
    }
}

pub fn prepare_workspace(
    guest_path: &str,
    program_src_dir: &str,
    program_toml_dir: &str,
    host_src_dir: &str,
    host_toml_dir: &str,
    base_host_toml_dir: &str,
    base_guest_toml_dir: &str,
) -> io::Result<()> {
    if let Err(e) = fs::remove_dir_all(program_src_dir) {
        if e.kind() != ErrorKind::NotFound {
            return Err(e);
        }
    }
    fs::create_dir_all(program_src_dir)?;
    fs::create_dir_all(host_src_dir)?;
    let program_path = format!("{}/src/", guest_path);
    copy_dir_all(&program_path, program_src_dir)?;
    copy_dir_all(&program_path, host_src_dir)?;
    fs::copy(base_guest_toml_dir, program_toml_dir)?;
    fs::copy(base_host_toml_dir, host_toml_dir)?;
    let toml_path = format!("{}/Cargo.toml", guest_path);
    copy_dependencies(&toml_path, program_toml_dir);
    copy_dependencies(&toml_path, host_toml_dir);

    Ok(())
}

// todo -> extract using regex
pub fn extract_till_last_occurence(
    file_path: &str,
    search: &str,
    start: &str,
    end: &str,
) -> io::Result<Option<String>> {
    let mut contents = String::new();
    fs::File::open(file_path)?.read_to_string(&mut contents)?;
    // Find the position of the search string
    if let Some(search_pos) = contents.find(search) {
        // Find the position of the first 'start' after the search string
        if let Some(start_pos) = contents[search_pos..].find(start) {
            let start_pos = search_pos + start_pos;
            // Find the position of the last 'end' after the opening 'start'
            if let Some(end_pos) = contents[start_pos..].rfind(end) {
                let end_pos = start_pos + end_pos;
                // Extract and return the substring between 'start' and the last occurence 'end'
                return Ok(Some(contents[start_pos + 1..end_pos].to_string()));
            }
        }
    }
    Ok(None)
}

//TODO: refactor this to eliminate the clone at each step.
pub fn get_imports(filename: &str) -> io::Result<String> {
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
                while let Some(line) = lines.next() {
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

// TODO: Abstract Regex
pub fn extract_regex(file_path: &str, regex: &str) -> io::Result<Vec<String>> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut values = Vec::new();
    let regex = Regex::new(&regex).unwrap();

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
pub fn remove_lines(file_path: &str, target: &str) -> io::Result<()> {
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
