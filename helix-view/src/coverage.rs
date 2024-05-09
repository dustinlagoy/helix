use quick_xml::de::from_reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub struct Coverage {
    pub files: HashMap<std::path::PathBuf, FileCoverage>,
}

pub struct FileCoverage {
    pub lines: HashMap<u32, bool>,
}

#[derive(Deserialize, Debug)]
struct RawCoverage {
    #[serde(rename = "@version")]
    version: String,
    sources: Sources,
    packages: Packages,
}

#[derive(Deserialize, Debug)]
struct Sources {
    source: Vec<Source>,
}

#[derive(Deserialize, Debug)]
struct Source {
    #[serde(rename = "$value")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct Packages {
    package: Vec<Package>,
}

#[derive(Deserialize, Debug)]
struct Package {
    #[serde(rename = "@name")]
    name: String,
    classes: Classes,
}

#[derive(Deserialize, Debug)]
struct Classes {
    class: Vec<Class>,
}

#[derive(Deserialize, Debug)]
struct Class {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@filename")]
    filename: String,
    lines: Lines,
}

#[derive(Deserialize, Debug)]
struct Lines {
    line: Vec<Line>,
}

#[derive(Deserialize, Debug)]
struct Line {
    #[serde(rename = "@number")]
    number: u32,
    #[serde(rename = "@hits")]
    hits: u32,
}

pub fn parse(path: std::path::PathBuf) -> Option<Coverage> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let tmp: RawCoverage = from_reader(reader).ok()?;
    Some(tmp.into())
}

impl From<RawCoverage> for Coverage {
    fn from(coverage: RawCoverage) -> Self {
        let mut files = HashMap::new();
        for package in coverage.packages.package {
            for class in package.classes.class {
                let mut lines = HashMap::new();
                for line in class.lines.line {
                    lines.insert(line.number - 1, line.hits > 0);
                }
                for source in &coverage.sources.source {
                    let path: std::path::PathBuf = [&source.name, &class.filename].iter().collect();
                    if path.exists() {
                        files.insert(path, FileCoverage { lines });
                        break;
                    }
                }
            }
        }
        Coverage { files }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;
    use std::path::PathBuf;
    static TEST_STRING: &str = r#"<?xml version="1.0" ?>
<coverage version="7.3.0" timestamp="4333222111000">
	<sources>
		<source>a_src</source>
	</sources>
	<packages>
        <package name="a package">
            <classes>
                <class name="a class" filename="file.ext">
                    <lines>
                        <line number="3" hits="1"/>
                        <line number="5" hits="0"/>
                    </lines>
                </class>
                <class name="another class" filename="other.ext">
                    <lines>
                        <line number="1" hits="0"/>
                        <line number="7" hits="1"/>
                    </lines>
                </class>
            </classes>
        </package>
    </packages>
</coverage>"#;

    #[test]
    fn test_deserialize_raw_coverage_from_string() {
        let result: RawCoverage = from_str(TEST_STRING).unwrap();
        println!("result is {:?}", result);
        assert_eq!(result.version, "7.3.0");
        assert_eq!(result.sources.source[0].name, "a_src");
        assert_eq!(result.packages.package[0].name, "a package");
        let class = &result.packages.package[0].classes.class[0];
        assert_eq!(class.name, "a class");
        assert_eq!(class.filename, "file.ext");
        assert_eq!(class.lines.line[0].number, 3);
        assert_eq!(class.lines.line[0].hits, 1);
        assert_eq!(class.lines.line[1].number, 5);
        assert_eq!(class.lines.line[1].hits, 0);
    }

    #[test]
    fn test_convert_raw_coverage_to_coverage() {
        let tmp: RawCoverage = from_str(TEST_STRING).unwrap();
        let result: Coverage = tmp.into();
        assert_eq!(result.files.len(), 2);
        let first = result.files.get(&PathBuf::from("a_src/file.ext")).unwrap();
        assert!(first.lines.get(&0).is_none());
        assert_eq!(first.lines.get(&3), Some(&true));
        assert_eq!(first.lines.get(&5), Some(&false));
        let second = result.files.get(&PathBuf::from("a_src/other.ext")).unwrap();
        assert!(second.lines.get(&3).is_none());
        assert_eq!(second.lines.get(&1), Some(&false));
        assert_eq!(second.lines.get(&7), Some(&true));
    }
}
