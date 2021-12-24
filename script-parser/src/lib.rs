use std::collections::HashMap;

use serde::{Serialize, Deserialize};

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {

//     }
// }

pub fn parse(script: &str) -> Result<Vec<Script>, Error> {
    let mut scripts = Vec::new();
    let mut aliases = HashMap::new();
    let mut lines = script.lines().enumerate();
    while let Some((line, text)) = lines.next() {
        let mut args = text.split_whitespace();
        if let Some(arg0) = args.next() {
            match arg0 {
                ".set" => {
                    let name = args.next().ok_or(Error::SetName(line))?;
                    let variable = args.next().ok_or(Error::SetVariable(line))?;
                    aliases.insert(name.to_owned(), variable.to_owned());
                }
                name => {
                    if name.trim().len() < 4 || name.starts_with('@') {
                        continue;
                    }
                    // let name = args.next().ok_or(Error::ScriptName(line))?;
                    let end = &name.get(name.len().saturating_sub(2)..);
                    if !end.map(|end| end.eq_ignore_ascii_case("::")).unwrap_or_default() {
                        return Err(Error::ScriptName(line));
                    }
                    let name = &name[..name.len() - 2];
                    let at = args.next();
                    if !at
                        .map(|at| at.eq_ignore_ascii_case("@"))
                        .unwrap_or_default()
                    {
                        return Err(Error::ScriptName(line));
                    }
                    let location = args.next().ok_or(Error::ScriptLocation(line))?;
                    let location = Location::from_str_radix(location, 16)
                        .map_err(|err| Error::MalformedLocation(err, line))?;
                    let mut commands = Vec::new();
                    'commands: loop {
                        match lines.next() {
                            Some((line, text)) => {
                                match text.trim() {
                                    "end" | "return" | "step_end" | ".endm" | "" => break 'commands,
                                    _ => (),
                                }
                                let command = text
                                    .split_whitespace()
                                    .next()
                                    .ok_or(Error::NoCommand(line))?;
                                match command {
                                    ".byte" | ".2byte" => break 'commands,
                                    _ => (),
                                }

                                let arguments = text
                                    .split(command)
                                    .last()
                                    .map(|arguments| {
                                        arguments
                                            .split(',')
                                            .map(str::trim)
                                            .map(|s| aliases.get(s).map(|s| s.as_str()).unwrap_or(s))
                                            .map(str::to_owned)
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                commands.push(Command {
                                    command: command.to_owned(),
                                    arguments,
                                });
                            }
                            None => return Err(Error::EndOfFile("script")),
                        }
                    }

                    let script = Script {
                        name: name.to_owned(),
                        location,
                        commands,
                    };

                    scripts.push(script);
                }
            }
        }
    }
    Ok(scripts)
}

pub fn parse_message_script(script: &str) -> Result<Vec<Message>, Error> {
    let mut messages = Vec::new();let mut lines = script.lines().enumerate();
    while let Some((line, text)) = lines.next() {
        let mut args = text.split_whitespace();
        if let Some(name) = args.next() {
            if name.trim().len() < 4 {
                continue;
            }
            // let name = args.next().ok_or(Error::ScriptName(line))?;
            let end = &name[name.len() - 2..];
            if !end.eq_ignore_ascii_case("::") {
                return Err(Error::ScriptName(line));
            }
            let name = &name[..name.len() - 2];
            let at = args.next();
            if !at
                .map(|at| at.eq_ignore_ascii_case("@"))
                .unwrap_or_default()
            {
                return Err(Error::ScriptName(line));
            }
            let location = args.next().ok_or(Error::ScriptLocation(line))?;
            let location = Location::from_str_radix(location, 16)
                .map_err(|err| Error::MalformedLocation(err, line))?;
            let mut message_pages = Vec::new();
            let mut message_lines = Vec::new();
            'message: loop {
                match lines.next() {
                    Some((line, text)) => {
                        let command = text
                            .split_whitespace()
                            .next()
                            .ok_or(Error::NoCommand(line))?;

                        let text = text
                            .split(command)
                            .last()
                            .ok_or(Error::NoArguments(line))?;

                        let text = text.trim();

                        let mut text = text.split("\"");
                        
                        text.next();

                        let text = text.next().ok_or(Error::NoArguments(line))?;

                        if text.contains("$") {
                            let line = text.replace('$', "");
                            message_lines.push(line);
                            message_pages.push(std::mem::take(&mut message_lines));
                            break 'message;
                        }

                        if text.contains("\\p") {
                            let line = text.replace("\\p", "");
                            message_lines.push(line);
                            message_pages.push(std::mem::take(&mut message_lines));
                        }

                        for terminator in ["\\n", "\\l"] {
                            if text.contains(terminator) {
                                let line = text.replace(terminator, "");
                                message_lines.push(line);
                            }
                        }

                    }
                    None => return Err(Error::EndOfFile("message")),
                }
            }

            let message = Message {
                name: name.to_owned(),
                location,
                text: message_pages,
            };

            messages.push(message);
        }
    }
    Ok(messages)
}

pub type Location = u32;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    pub location: Location,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub command: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub name: String,
    pub location: Location,
    pub text: Vec<Vec<String>>,
}

#[derive(Debug)]
pub enum Error {
    SetName(usize),
    SetVariable(usize),
    ScriptName(usize),
    ScriptLocation(usize),
    MalformedLocation(std::num::ParseIntError, usize),
    NoCommand(usize),
    NoArguments(usize),
    EndOfFile(&'static str),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SetName(line) => {
                write!(f, "Could not get variable name for .set at line {}", line)
            }
            Error::SetVariable(line) => {
                write!(f, "Could not get variable for .set at line {}", line)
            }
            Error::MalformedLocation(err, line) => write!(
                f,
                "Could not parse script location at line {} with error {}",
                line, err
            ),
            _ => std::fmt::Debug::fmt(&self, f),
        }
    }
}
