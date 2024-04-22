use std::fs;
use std::io;
use std::path::Path;

fn read_functions_from_file(file_path: &str) -> Result<String, io::Error> {
    let content = fs::read_to_string(file_path)?;
    Ok(content)
}

fn choose_function(content: &str) -> Result<String, io::Error> {
    let mut functions = vec![];
    let mut i = 1;
    
    for line in content.lines() {
        if !line.trim().is_empty() {
            functions.push(line.trim());
            println!("{}. {}", i, line.trim());
            i += 1;
        }
    }

    println!("Choose a function to interact with:");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    let choice: usize = choice.trim().parse()?;
    if choice > 0 && choice <= functions.len() {
        Ok(functions[choice - 1].to_string())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid choice!"))
    }
}

fn count_parameters(signature: &str) -> usize {
    // Split the signature by commas and count the parts
    signature.split(',').count()
}

fn extract_params(signature: &str) -> Vec<&str> {
    // Remove function name and parentheses
    let params = signature.trim().split('(').nth(1).unwrap_or("");
    let params = params.trim_end_matches(')');
    // Split parameters by commas and collect them into a vector
    params.split(',').map(|param| param.trim()).collect()
}

fn generate_interaction_code(function_name: &str, args: Vec<&str>) -> String {
    const COMMANDS_PROVE: &str = r#"Commands::ProveJolt({args})"#;
    const HOST_MAIN: &str = r#"pub fn main() {{
        let (prove_{func}, verify_{func}) = guest::build_{func}();

        let ({param_decls}) = ({params});
        let (output, proof) = {commands_prove};
        let is_valid = verify_{func}(proof);

        println!("output: {{}}", output);
        println!("valid: {{}}", is_valid);
    }}"#;

    let num_params = args.len();
    let param_decls = args.join(", ");
    let params = args.iter().map(|param| format!("{}_param", param)).collect::<Vec<_>>().join(", ");

    let commands_prove = COMMANDS_PROVE.replace("{args}", &format!("({})", params));
    let host_main = HOST_MAIN
        .replace("{func}", function_name)
        .replace("{param_decls}", &param_decls)
        .replace("{params}", &params)
        .replace("{commands_prove}", &commands_prove);

    host_main
}

fn main() -> Result<(), io::Error> {
    // Specify the directory where your function files are stored
    let directory = "functions";

    // Read all files in the specified directory
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_path = path.to_str().unwrap();
            println!("File: {}", file_path);
            let content = read_functions_from_file(file_path)?;

            match choose_function(&content) {
                Ok(chosen_function) => {
                    // Generate interaction code based on the chosen function
                    println!("Chosen function: {}", chosen_function);

                    // Get the function name and parameters from the chosen function
                    let parts: Vec<&str> = chosen_function.split_whitespace().collect();
                    let function_name = parts[0];
                    let signature = parts[1..].join(" ");
                    let num_params = count_parameters(&signature);
                    let params = extract_params(&signature);

                    println!("Function signature: {}", signature);
                    println!("Number of parameters: {}", num_params);
                    println!("Parameters: {:?}", params);

                    // Generate interaction code
                    let interaction_code = generate_interaction_code(function_name, params);

                    println!("{}", interaction_code);
                }
                Err(err) => println!("Error: {}", err),
            }
        }
    }

    Ok(())
}
