fn main() {
    let text = std::fs::read_to_string("scripts.inc").unwrap();
    match script_parser::parse(&text) {
        Ok(script) => println!("{:#?}", script),
        Err(err) => eprintln!("Could not parse script with error {}", err),
    }
    let text = std::fs::read_to_string("text.inc").unwrap();
    match script_parser::parse_message_script(&text) {
        Ok(script) => println!("{:#?}", script),
        Err(err) => eprintln!("Could not parse script with error {}", err),
    }
}
