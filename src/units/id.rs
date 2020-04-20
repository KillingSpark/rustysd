use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum UnitIdKind {
    Target,
    Socket,
    Service,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct UnitId {
    pub kind: UnitIdKind,
    pub name: String,
}
impl UnitId {
    pub fn name_without_suffix(&self) -> String {
        let split: Vec<_> = self.name.split('.').collect();
        split[0..split.len() - 1].join(".")
    }
}

impl fmt::Debug for UnitId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("{}", self.name).as_str())
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("{:?}", self).as_str())
    }
}

impl std::cmp::PartialOrd for UnitId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl std::cmp::Ord for UnitId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq<str> for UnitId {
    fn eq(&self, other: &str) -> bool {
        self.name.eq(other)
    }
}
impl PartialEq<String> for UnitId {
    fn eq(&self, other: &String) -> bool {
        self.name.eq(other)
    }
}
impl PartialEq<dyn AsRef<str>> for UnitId {
    fn eq(&self, other: &dyn AsRef<str>) -> bool {
        self.name.eq(other.as_ref())
    }
}
