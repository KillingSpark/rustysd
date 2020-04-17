use crate::units::*;
use std::path::PathBuf;

pub fn parse_target(
    parsed_file: ParsedFile,
    path: &PathBuf,
    chosen_id: UnitId,
) -> Result<ParsedTargetConfig, ParsingErrorReason> {
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Unit]" => {
                unit_config = Some(parse_unit_section(section, path)?);
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section)?);
            }
            _ => return Err(ParsingErrorReason::UnknownSection(name.to_owned())),
        }
    }

    let conf = match unit_config {
        Some(conf) => conf,
        None => {
            return Err(ParsingErrorReason::SectionNotFound("Unit".to_owned()));
        }
    };

    Ok(ParsedTargetConfig {
        common: ParsedCommonConfig {
            unit: conf,
            install: install_config.unwrap_or_else(Default::default),
        },
    })
}
