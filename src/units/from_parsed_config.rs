use crate::units::*;

pub fn unit_from_parsed_service(conf: ParsedServiceConfig) -> Unit {
    unimplemented!();
}

pub fn unit_from_parsed_socket(conf: ParsedSocketConfig) -> Unit {
    unimplemented!();
}
pub fn unit_from_parsed_target(conf: ParsedTargetConfig) -> Unit {
    unimplemented!();
}

impl std::convert::From<ParsedServiceConfig> for Unit {
    fn from(conf: ParsedServiceConfig) -> Unit {
        unit_from_parsed_service(conf)
    }
}
impl std::convert::From<ParsedSocketConfig> for Unit {
    fn from(conf: ParsedSocketConfig) -> Unit {
        unit_from_parsed_socket(conf)
    }
}
impl std::convert::From<ParsedTargetConfig> for Unit {
    fn from(conf: ParsedTargetConfig) -> Unit {
        unit_from_parsed_target(conf)
    }
}
