use glob::glob;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{env, fmt};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

pub enum Language {
    Rust,
    Java,
}

impl Language {
    fn get_folder(&self) -> Result<String, String> {
        match self {
            Language::Rust => {
                let home = match env::var("HOME") {
                    Ok(h) => h,
                    Err(e) => return Err(e.to_string()),
                };

                Ok(String::from(&format!("{home}/.cargo/registry/src/**/*.rs")))
            }
            Language::Java => {
                let java_path = match env::var("JAVA_LINES") {
                    Ok(j) => j,
                    Err(e) => return Err(e.to_string()),
                };
                Ok(String::from(java_path))
            }
        }
    }

    fn get_paths(&self) -> Result<Vec<String>, LinesError> {
        let result = glob(&self.get_folder().unwrap())
            .unwrap()
            .map(|p| p.unwrap().display().to_string())
            .collect();

        Ok(result)
    }
}

pub struct LineConfig {
    pub language: Language,
}

#[derive(Debug)]
struct LinesError(String);

impl fmt::Display for LinesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for LinesError {}

pub fn get_random_line(config: &LineConfig) -> Result<String, Box<dyn Error>> {
    let file = File::open(get_random_file_path(&config)?)?;
    let lines = filter_code_lines(&config, get_lines_from_file(file));
    match get_random_string(lines) {
        Some(s) => Ok(s),
        None => Err(Box::new(LinesError("Error getting random line.".into()))),
    }
}

fn get_lines_from_file(file: File) -> Vec<String> {
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
        .collect();
    lines
}

fn get_random_file_path(config: &LineConfig) -> Result<String, Box<dyn Error>> {
    match get_random_string(config.language.get_paths()?) {
        Some(s) => Ok(s),
        None => Err(Box::new(LinesError(
            "Error getting random file path.".into(),
        ))),
    }
}

fn filter_code_lines(config: &LineConfig, lines: Vec<String>) -> Vec<String> {
    match config.language {
        Language::Rust => lines
            .into_iter()
            .filter(|l| !l.contains('/') && l.len() > 10)
            .map(|l| l.trim().to_string())
            .collect(),
        Language::Java => lines,
    }
}

fn get_random_string(lines: Vec<String>) -> Option<String> {
    Some(lines.choose(&mut thread_rng())?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_code_lines_vec() -> Vec<String> {
        vec![
            "///".to_string(),
            "let thing".to_string(),
            "         let thing = do_this_long_thing(hello)".to_string(),
        ]
    }

    #[test]
    fn test_filter_code_lines() {
        let config = LineConfig {
            language: Language::Rust,
        };
        let result = filter_code_lines(&config, get_test_code_lines_vec());
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get(0).unwrap(),
            "let thing = do_this_long_thing(hello)"
        );
    }

    #[test]
    fn test_get_random_line_one_line() {
        let result = get_random_string(vec![String::from("random")]);
        assert_eq!(result, Some(String::from("random")));
    }

    #[test]
    fn test_get_random_line_no_lines() {
        let result = get_random_string(vec![]);
        assert_eq!(result, None);
    }
}
