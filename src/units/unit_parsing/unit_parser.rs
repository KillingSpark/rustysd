//! Parse all supported unit types / options for these and do needed operations like matching services <-> sockets and adding implicit dependencies like
//! all sockets to socket.target

use crate::units::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub type ParsedSection = HashMap<String, Vec<(u32, String)>>;
pub type ParsedFile = HashMap<String, ParsedSection>;

pub fn parse_file(content: &str) -> Result<ParsedFile, ParsingError> {
    let mut sections = HashMap::new();
    let lines: Vec<&str> = content.split('\n').collect();
    let lines: Vec<_> = lines.iter().map(|s| s.trim()).collect();

    let mut lines_left = &lines[..];

    // remove lines before the first section
    while !lines_left.is_empty() && !lines_left[0].starts_with('[') {
        lines_left = &lines_left[1..];
    }
    let mut current_section_name: String = lines_left[0].into();
    let mut current_section_lines = Vec::new();

    lines_left = &lines_left[1..];

    while !lines_left.is_empty() {
        let line = lines_left[0];

        if line.starts_with('[') {
            if sections.contains_key(&current_section_name) {
                return Err(ParsingError::from(format!(
                    "Section {} occured mutliple times",
                    current_section_name
                )));
            } else {
                sections.insert(
                    current_section_name.clone(),
                    parse_section(&current_section_lines),
                );
            }
            current_section_name = line.into();
            current_section_lines.clear();
        } else {
            current_section_lines.push(line);
        }
        lines_left = &lines_left[1..];
    }

    // insert last section
    if sections.contains_key(&current_section_name) {
        return Err(ParsingError::from(format!(
            "Section {} occured mutliple times",
            current_section_name
        )));
    } else {
        sections.insert(current_section_name, parse_section(&current_section_lines));
    }

    Ok(sections)
}

pub fn map_tupels_to_second<X, Y: Clone>(v: Vec<(X, Y)>) -> Vec<Y> {
    v.iter().map(|(_, scnd)| scnd.clone()).collect()
}

pub fn string_to_bool(s: &str) -> bool {
    let s_upper = &s.to_uppercase();
    let c: char = s_upper.chars().nth(0).unwrap();

    let is_num_and_one = s.len() == 1 && c == '1';
    *s_upper == *"YES" || *s_upper == *"TRUE" || is_num_and_one
}

pub fn parse_unit_section(mut section: ParsedSection, path: &PathBuf) -> UnitConfig {
    let wants = section.remove("WANTS");
    let requires = section.remove("REQUIRES");
    let after = section.remove("AFTER");
    let before = section.remove("BEFORE");
    let description = section.remove("DESCRIPTION");

    if !section.is_empty() {
        panic!(
            "Unit section has unrecognized/unimplemented options: {:?}",
            section
        );
    }

    UnitConfig {
        filepath: path.clone(),
        description: description.map(|x| (x[0]).1.clone()).unwrap_or_default(),
        wants: map_tupels_to_second(wants.unwrap_or_default()),
        requires: map_tupels_to_second(requires.unwrap_or_default()),
        after: map_tupels_to_second(after.unwrap_or_default()),
        before: map_tupels_to_second(before.unwrap_or_default()),
    }
}

pub fn parse_install_section(mut section: ParsedSection) -> InstallConfig {
    let wantedby = section.remove("WANTEDBY");
    let requiredby = section.remove("REQUIREDBY");

    if !section.is_empty() {
        panic!(
            "Install section has unrecognized/unimplemented options: {:?}",
            section
        );
    }

    InstallConfig {
        wanted_by: map_tupels_to_second(wantedby.unwrap_or_default()),
        required_by: map_tupels_to_second(requiredby.unwrap_or_default()),
    }
}

pub fn get_file_list(path: &PathBuf) -> Result<Vec<std::fs::DirEntry>, String> {
    if !path.exists() {
        return Err(format!("Path to services does not exist: {:?}", path));
    }
    if !path.is_dir() {
        return Err(format!("Path to services does not exist: {:?}", path));
    }
    let mut files: Vec<_> = match std::fs::read_dir(path) {
        Ok(iter) => {
            let files_vec = iter.fold(Ok(Vec::new()), |acc, file| {
                if let Ok(mut files) = acc {
                    match file {
                        Ok(f) => {
                            files.push(f);
                            Ok(files)
                        }
                        Err(e) => Err(format!("Couldnt read dir entry: {}", e)),
                    }
                } else {
                    acc
                }
            });
            match files_vec {
                Ok(files) => files,
                Err(e) => return Err(e),
            }
        }
        Err(e) => return Err(format!("Error while reading dir: {}", e)),
    };
    files.sort_by(|l, r| l.path().cmp(&r.path()));

    Ok(files)
}

pub fn parse_section(lines: &[&str]) -> ParsedSection {
    let mut entries: ParsedSection = HashMap::new();

    let mut entry_number = 0;
    for line in lines {
        //ignore comments
        if line.starts_with('#') {
            continue;
        }

        //check if this is a key value pair
        let pos = if let Some(pos) = line.find(|c| c == '=') {
            pos
        } else {
            continue;
        };
        let (name, value) = line.split_at(pos);

        let value = value.trim_start_matches('=');
        let value = value.trim();
        let name = name.trim().to_uppercase();
        let values: Vec<String> = value.split(',').map(|x| x.into()).collect();

        let vec = entries.entry(name).or_insert_with(Vec::new);
        for value in values {
            vec.push((entry_number, value));
            entry_number += 1;
        }
    }

    entries
}
