
use std::{fmt::format, io::Write};

pub fn gen_examples(path: &str, func_names: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // write a stirng to a file

    println!("function name in examples are {:?}", func_names);
    let mut example = String::new();
    
    
    
    for func_name in func_names {
        // the if only has two paths because right now there is only add and get functions 
        // will need to fix for update and delete
        if func_name.contains("add") {
            // print struct 
            println!("struct is {}", "struct name");
            example.push_str(format!("
            fetch(\"http://localhost:3002/api/{}\", {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json'
                }},
                body: JSON.stringify({{
                    {}
                }})
            }}).then(response => response.json()).then(data => console.log(data)); 
            ", func_name, String::from("[ add object of key values based on the struct ]")).as_str());
        } else {
            example.push_str(format!("
            fetch(\"http://localhost:3002/api/{}\").then(response => response.json()).then(data => console.log(data));
            ", func_name).as_str());
        }
        example.push_str("\n");
    }

    println!("");
    println!("");
    println!("example string is {}", example);
    println!("");
    println!("");
    println!("attempting to write to {}\n", "../".to_owned() + path + "/examples.js");
    println!("");
    println!("");
    let mut file = std::fs::File::create("../".to_owned() + path + "/examples.js")?;
    file.write_all(example.as_bytes())?;
    println!("");
    println!("");
    Ok(())
}