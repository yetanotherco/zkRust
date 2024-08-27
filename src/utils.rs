use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Read, Seek, Write},
    path::Path,
};

pub fn add_before_substring(original_string: &str, substring: &str, text_to_add: &str) -> String {
    if let Some(index) = original_string.find(substring) {
        let mut modified_string = String::with_capacity(original_string.len() + text_to_add.len());
        modified_string.push_str(&original_string[..index]);
        modified_string.push_str(text_to_add);
        modified_string.push_str(&original_string[index..]);
        modified_string
    } else {
        original_string.to_string()
    }
}

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
    let mut file = fs::File::create(&file_path)?;
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
    fs::File::open(&target_file)?.read_to_string(&mut target_contents)?;

    // Find the position of the search string in the target file
    if let Some(pos) = target_contents.find(search_string) {
        // Split the target contents into two parts
        let (before, after) = target_contents.split_at(pos + search_string.len());

        // Combine the parts with the insert contents
        let new_contents = format!("{}{}{}", before, text, after);

        // Write the new contents back to the target file
        let mut file = fs::File::create(&target_file)?;
        file.write_all(new_contents.as_bytes())?;
    } else {
        println!("Search string not found in target file.");
    }

    Ok(())
}

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
    workspace_program_src_dir: &str,
    workspace_program_toml_dir: &str,
    base_toml_dir: &str,
) -> io::Result<()> {
    if let Err(e) = fs::remove_dir_all(&workspace_program_src_dir) {
        if e.kind() != ErrorKind::NotFound {
            return Err(e);
        }
    }
    fs::create_dir_all(workspace_program_src_dir)?;
    let workspace_program_path = format!("{}/src/", guest_path);
    copy_dir_all(&workspace_program_path, workspace_program_src_dir).unwrap();
    fs::copy(base_toml_dir, workspace_program_toml_dir).unwrap();
    let toml_path = format!("{}/Cargo.toml", guest_path);
    copy_dependencies(&toml_path, workspace_program_toml_dir);

    Ok(())
}
