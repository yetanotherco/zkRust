use std::{
    fs::{self, OpenOptions},
    io::{self, ErrorKind, Read, Seek, Write},
    path::Path,
};

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
