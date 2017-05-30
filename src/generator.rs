use std::fs;
use std::io::{self, Write, Read, Seek, SeekFrom};
use heck::{CamelCase, SnekCase};

pub fn generate_resource(name: &str) -> io::Result<()> {
    let name_snake = name.to_snek_case();
    let name_camel = name.to_camel_case();

    // create resource
    let mut file = fs::File::create(format!("src/resources/{}.rs", name_snake))?;
    file.write(resource_file(&name_camel).as_bytes())?;

    // add re-export
    let mut file = fs::OpenOptions::new().append(true).open("src/resources/mod.rs")?;
    file.write(format!("\nmod {};\npub use self::{}::{};\n", name_snake, name_snake, name_camel).as_bytes())?;

    // edit routes! macro
    let mut file = fs::OpenOptions::new().read(true).write(true).open("src/routing.rs")?;
    let resource_obj = format!("    resource {} {{", name_camel);
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    let mut lines = string.lines().collect::<Vec<_>>();
    let (routes_pos, pos) = get_pos(&lines);
    if routes_pos + 1 == pos {
        lines.insert(pos, &resource_obj);
        lines.insert(pos + 1, "    }");
    } else {
        lines.insert(pos, "");
        lines.insert(pos + 1, &resource_obj);
        lines.insert(pos + 2, "    }");
    }

    file.seek(SeekFrom::Start(0))?;
    file.write_all(lines.join("\n").as_bytes())?;
    Ok(())
}


fn resource_file(name: &str) -> String {
    format!("\
use cargonauts::*;

pub struct {} {{
}}

impl Resource for {} {{
    type Identifier = (); // TODO: Select the identifier type for this resource
}}
", name, name)
}

fn get_pos(lines: &[&str]) -> (usize, usize) {
    let mut routes = 0;
    let mut ctr = 0;
    let mut in_routes = false;
    for line in lines {
        if in_routes {
            if line.starts_with("}") { return (routes, ctr) }
        } else if line.starts_with("routes!") {
            in_routes = true;
            routes = ctr;
        }
        ctr += 1;
    }
    panic!("No routes macro in routing.rs");
}
