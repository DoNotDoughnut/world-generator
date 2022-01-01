use hashbrown::HashMap;

use serde::{Serialize, Deserialize};

use super::TrainerError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrainerPokemon {
    pub ivs: u8,
    pub level: u8,
    pub species: String,
    pub moves: Option<Vec<String>>,
    pub item: Option<String>,
}

pub fn parse_parties(file: &str) -> Result<HashMap<String, Vec<TrainerPokemon>>, TrainerError> {

    enum State {
        File,
        Party,
        Pokemon,
    }

    let mut lines = file.lines().enumerate();

    let mut state = State::File;
    let mut trainers = HashMap::new();
    let mut mons = Vec::new();
    let mut current_trainer = None;
    let mut current_pokemon = None;

    while let Some((line, text)) = lines.next() {
        match state {
            State::File => match text.trim() {
                "" => continue,
                text => {
                    let mut words = text.split_ascii_whitespace().skip(4);
                    current_trainer = words.next().map(|s| s[..s.len() - 2].to_owned());
                    if current_trainer.is_some() {
                        state = State::Party;
                    }
                }
            },
            State::Party => match text.trim() {
                "};" => {
                    let pokemon = std::mem::take(&mut mons);
                    let name = current_trainer.take().unwrap();

                    trainers.insert(name, pokemon);

                    state = State::File;
                }
                _ => state = State::Pokemon,
            },
            State::Pokemon => {
                let pokemon = current_pokemon.get_or_insert(TrainerPokemon::default());
                match text.trim() {
                    "}," | "}" => {
                        if let Some(current) = current_pokemon.take() {
                            mons.push(current);
                        }
                        state = State::Party;
                    }
                    _ =>  {
                        let (left, right) = text.split_once('=').ok_or_else(|| TrainerError::FieldParse(line, text.to_owned()))?;
                        let (left, right) = (left.trim(), right.trim());
                        let right = &right[..right.len() - 1];
                        match left {
                            ".iv" => {
                                pokemon.ivs = right
                                    .parse()
                                    .map_err(|err| TrainerError::NumParse(line, "ivs", err))?
                            }
                            ".lvl" => {
                                pokemon.level = right
                                    .parse()
                                    .map_err(|err| TrainerError::NumParse(line, "level", err))?
                            }
                            ".species" => pokemon.species = right.to_owned(),
                            ".moves" => {
                                let (lb, ..) =
                                    right
                                        .char_indices()
                                        .find(|(.., c)| c == &'{')
                                        .ok_or_else(|| TrainerError::BracketParse(line, "moves"))?;
                                let (rb, ..) =
                                    right
                                        .char_indices()
                                        .find(|(.., c)| c == &'}')
                                        .ok_or_else(|| TrainerError::BracketParse(line, "moves"))?;
                                let array = &right[lb + 1..rb];
                                pokemon.moves = Some(array.split(',').map(str::trim).map(str::to_owned).collect());
                            },
                            ".heldItem" => pokemon.item = Some(right.to_owned()),
                            field => return Err(TrainerError::UnknownField(line, field.to_owned())),
                        }
                    },
                }
            }
        }
    }

    Ok(trainers)
}