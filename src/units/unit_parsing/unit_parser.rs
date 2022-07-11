//! Parse all supported unit types / options for these and do needed operations like matching services <-> sockets and adding implicit dependencies like
//! all sockets to socket.target

use log::debug;

use crate::units::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub type ParsedSection = HashMap<String, Vec<(u32, String)>>;
pub type ParsedFile = HashMap<String, ParsedSection>;

pub fn parse_file(content: &str) -> Result<ParsedFile, ParsingErrorReason> {
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
                return Err(ParsingErrorReason::SectionTooOften(
                    current_section_name.to_owned(),
                ));
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
        return Err(ParsingErrorReason::SectionTooOften(
            current_section_name.to_owned(),
        ));
    } else {
        sections.insert(current_section_name, parse_section(&current_section_lines));
    }

    Ok(sections)
}

pub fn map_tupels_to_second<X, Y: Clone>(v: Vec<(X, Y)>) -> Vec<Y> {
    v.iter().map(|(_, scnd)| scnd.clone()).collect()
}

pub fn string_to_bool(s: &str) -> bool {
    if s.len() == 0 {
        return false;
    }

    let s_upper = &s.to_uppercase();
    let c: char = s_upper.chars().nth(0).unwrap();

    let is_num_and_one = s.len() == 1 && c == '1';
    *s_upper == *"YES" || *s_upper == *"TRUE" || is_num_and_one
}

fn parse_environment(raw_line: &str) -> Result<EnvVars, ParsingErrorReason> {
    debug!("raw line: {}", raw_line);
    let split = shlex::split(raw_line).ok_or(ParsingErrorReason::Generic(format!(
        "Could not parse cmdline: {}",
        raw_line
    )))?;
    debug!("split: {:?}", split);
    let mut vars: Vec<(String, String)> = Vec::new();

    for pair in split {
        let p: Vec<&str> = pair.split('=').collect();
        let key = p[0].to_owned();
        let val = p[1].to_owned();
        vars.push((key, val));
    }

    Ok(EnvVars { vars })
}

pub fn parse_unit_section(
    mut section: ParsedSection,
) -> Result<ParsedUnitSection, ParsingErrorReason> {
    let wants = section.remove("WANTS");
    let requires = section.remove("REQUIRES");
    let after = section.remove("AFTER");
    let before = section.remove("BEFORE");
    let description = section.remove("DESCRIPTION");

    if !section.is_empty() {
        return Err(ParsingErrorReason::UnusedSetting(
            section.keys().next().unwrap().to_owned(),
        ));
    }

    Ok(ParsedUnitSection {
        description: description.map(|x| (x[0]).1.clone()).unwrap_or_default(),
        wants: map_tupels_to_second(wants.unwrap_or_default()),
        requires: map_tupels_to_second(requires.unwrap_or_default()),
        after: map_tupels_to_second(after.unwrap_or_default()),
        before: map_tupels_to_second(before.unwrap_or_default()),
    })
}

fn make_stdio_option(setting: &str) -> Result<StdIoOption, ParsingErrorReason> {
    if setting.starts_with("file:") {
        let p = setting.trim_start_matches("file:");
        Ok(StdIoOption::File(p.into()))
    } else if setting.starts_with("append:") {
        let p = setting.trim_start_matches("append:");
        Ok(StdIoOption::AppendFile(p.into()))
    } else {
        return Err(ParsingErrorReason::UnsupportedSetting(format!(
            "StandardOutput: {}",
            setting
        )));
    }
}

pub fn parse_exec_section(
    section: &mut ParsedSection,
) -> Result<ParsedExecSection, ParsingErrorReason> {
    let user = section.remove("USER");
    let group = section.remove("GROUP");
    let stdout = section.remove("STANDARDOUTPUT");
    let stderr = section.remove("STANDARDERROR");
    let supplementary_groups = section.remove("SUPPLEMENTARYGROUPS");
    let environment = section.remove("ENVIRONMENT");

    let user = match user {
        None => None,
        Some(mut vec) => {
            if vec.len() == 1 {
                Some(vec.remove(0).1)
            } else if vec.len() > 1 {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "User".into(),
                    super::map_tupels_to_second(vec),
                ));
            } else {
                None
            }
        }
    };

    let group = match group {
        None => None,
        Some(mut vec) => {
            if vec.len() == 1 {
                Some(vec.remove(0).1)
            } else if vec.len() > 1 {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "Group".into(),
                    super::map_tupels_to_second(vec),
                ));
            } else {
                None
            }
        }
    };
    let stdout_path = match stdout {
        None => None,
        Some(mut vec) => {
            if vec.len() == 1 {
                Some(vec.remove(0).1)
            } else if vec.len() > 1 {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "Standardoutput".into(),
                    super::map_tupels_to_second(vec),
                ));
            } else {
                None
            }
        }
    };
    let stdout_path = if let Some(p) = stdout_path {
        Some(make_stdio_option(&p)?)
    } else {
        None
    };

    let stderr_path = match stderr {
        None => None,
        Some(mut vec) => {
            if vec.len() == 1 {
                Some(vec.remove(0).1)
            } else if vec.len() > 1 {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "Standarderror".into(),
                    super::map_tupels_to_second(vec),
                ));
            } else {
                None
            }
        }
    };
    let stderr_path = if let Some(p) = stderr_path {
        Some(make_stdio_option(&p)?)
    } else {
        None
    };

    let supplementary_groups = match supplementary_groups {
        None => Vec::new(),
        Some(vec) => vec.iter().fold(Vec::new(), |mut acc, (_id, list)| {
            acc.extend(list.split(' ').map(|x| x.to_string()));
            acc
        }),
    };

    let environment = match environment {
        Some(vec) => {
            debug!("Env vec: {:?}", vec);
            Some(parse_environment(&vec[0].1)?)
        }
        None => None,
    };

    Ok(ParsedExecSection {
        user,
        group,
        stderr_path,
        stdout_path,
        supplementary_groups,
        environment,
    })
}

pub fn parse_install_section(
    mut section: ParsedSection,
) -> Result<ParsedInstallSection, ParsingErrorReason> {
    let wantedby = section.remove("WANTEDBY");
    let requiredby = section.remove("REQUIREDBY");

    if !section.is_empty() {
        return Err(ParsingErrorReason::UnusedSetting(
            section.keys().next().unwrap().to_owned(),
        ));
    }

    Ok(ParsedInstallSection {
        wanted_by: map_tupels_to_second(wantedby.unwrap_or_default()),
        required_by: map_tupels_to_second(requiredby.unwrap_or_default()),
    })
}

pub fn get_file_list(path: &PathBuf) -> Result<Vec<std::fs::DirEntry>, ParsingErrorReason> {
    if !path.exists() {
        return Err(ParsingErrorReason::Generic(format!(
            "Path to services does not exist: {:?}",
            path
        )));
    }
    if !path.is_dir() {
        return Err(ParsingErrorReason::Generic(format!(
            "Path to services does not exist: {:?}",
            path
        )));
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
                        Err(e) => Err(e),
                    }
                } else {
                    acc
                }
            });
            match files_vec {
                Ok(files) => files,
                Err(e) => return Err(ParsingErrorReason::FileError(Box::new(e))),
            }
        }
        Err(e) => return Err(ParsingErrorReason::FileError(Box::new(e))),
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
