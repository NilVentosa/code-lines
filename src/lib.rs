use glob::glob;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{env, fmt};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

type LinesResult<T> = Result<T, LinesError>;

pub enum Language {
    Rust,
    Java,
}

impl Language {
    fn default_folder(&self) -> Option<String> {
        let home = match env::var("HOME") {
            Ok(h) => h,
            Err(_) => return None,
        };

        match self {
            Language::Rust => Some(String::from(&format!("{home}/.cargo/registry/src/**/*.rs"))),
            Language::Java => None,
        }
    }

    fn env_var_folder(&self) -> Option<String> {
        match self {
            Language::Rust => match env::var("RUST_LINES") {
                Ok(folder) => return Some(format!("{folder}/**/*.rust")),
                Err(_) => None,
            },
            Language::Java => match env::var("JAVA_LINES") {
                Ok(folder) => return Some(format!("{folder}/**/*.java")),
                Err(_) => None,
            },
        }
    }

    fn folder(&self) -> Option<String> {
        if self.env_var_folder().is_some() {
            return self.env_var_folder();
        }
        if self.default_folder().is_some() {
            return self.default_folder();
        }
        None
    }

    fn get_paths(&self) -> LinesResult<Vec<String>> {
        if let Some(folder) = &self.folder() {
            if let Ok(paths) = glob(folder) {
                return Ok(paths
                    .filter_map(Result::ok)
                    .map(|p| p.display().to_string())
                    .collect());
            };
        }
        Err(LinesError(format!(
            "Error getting file paths for {}.",
            self
        )))
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Java => write!(f, "Java"),
        }
    }
}

pub struct LineConfig {
    pub language: Language,
}

#[derive(Debug)]
pub struct LinesError(String);

impl fmt::Display for LinesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for LinesError {}

///
/// Returns a random line of code that matches de config argument
/// It returns a [`LinesError`] if none is found
///
/// # Arguments
///
/// * `config` - A reference to a [`LineConfig`]
///
pub fn get_random_line(config: &LineConfig) -> LinesResult<String> {
    match File::open(get_random_file_path(config)?) {
        Ok(file) => get_random_string(&filter_code_lines(config, get_lines_from_file(file))),
        Err(e) => Err(LinesError(e.to_string())),
    }
}

fn get_lines_from_file(file: File) -> Vec<String> {
    BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .collect()
}

fn get_random_file_path(config: &LineConfig) -> LinesResult<String> {
    get_random_string(&config.language.get_paths()?)
}

fn filter_code_lines(config: &LineConfig, lines: Vec<String>) -> Vec<String> {
    match config.language {
        Language::Rust => lines
            .into_iter()
            .filter(|l| !l.contains('/') && l.len() > 10)
            .map(|l| l.trim().to_string())
            .collect(),
        Language::Java => lines
            .into_iter()
            .filter(|l| !l.contains('/') && l.len() > 10)
            .map(|l| l.trim().to_string())
            .collect(),
    }
}

fn get_random_string(lines: &Vec<String>) -> LinesResult<String> {
    match lines.choose(&mut thread_rng()) {
        Some(line) => Ok(line.to_string()),
        None => Err(LinesError(String::from("Error getting random string."))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_rust_code_lines() {
        let code_lines = vec![
            "///".to_string(),
            "let thing".to_string(),
            "         let thing = do_this_long_thing(hello)".to_string(),
        ];

        let config = LineConfig {
            language: Language::Rust,
        };

        let result = filter_code_lines(&config, code_lines);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get(0).unwrap(),
            "let thing = do_this_long_thing(hello)"
        );
    }

    #[test]
    fn test_get_random_string_one_string() {
        let result = get_random_string(&vec![String::from("random")]);
        assert_eq!(result.unwrap(), String::from("random"));
    }

    #[test]
    fn test_get_random_string_no_strings() {
        let result = get_random_string(&vec![]);
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_get_random_string_various_strings() {
        let thing = vec![String::from("o"), String::from("a")];
        let result = get_random_string(&thing);
        assert_eq!(true, thing.contains(&result.unwrap()));
    }
}
