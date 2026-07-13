//! Quiz data model and loading from TOML files.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::Error;

/// One quiz topic loaded from a TOML file.
#[derive(Debug, Deserialize)]
pub struct Quiz {
    /// Human-readable topic name (English).
    pub title: String,
    /// Group shown on the index page.
    pub section: String,
    /// Interaction kind shared by every question.
    pub kind: Kind,
    /// Short intro shown above the questions.
    #[serde(default)]
    pub intro: String,
    /// Questions in display order.
    #[serde(default, rename = "question")]
    pub questions: Vec<Question>,
}

/// How a question is answered.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Several correct options (checkboxes).
    Multi,
    /// Exactly one correct option (radio).
    Single,
    /// Free text matched against `answers`.
    Text,
}

/// A single question.
#[derive(Debug, Deserialize)]
pub struct Question {
    /// Stable, unique id within the quiz.
    pub id: u32,
    /// Question text with a `___` blank.
    pub prompt: String,
    /// Explanation shown after answering (may contain inline HTML).
    #[serde(default)]
    pub explain: String,
    /// Options for `multi` and `single` kinds.
    #[serde(default)]
    pub options: Vec<Opt>,
    /// Accepted answers for the `text` kind (matched case-insensitively).
    #[serde(default)]
    pub answers: Vec<String>,
}

/// One option of a `multi` or `single` question.
#[derive(Debug, Deserialize)]
pub struct Opt {
    /// Option text.
    pub text: String,
    /// Whether this option is correct.
    pub correct: bool,
}

/// All quizzes, keyed by id (the file stem).
#[derive(Debug)]
pub struct Quizzes {
    items: BTreeMap<String, Quiz>,
}

impl Quizzes {
    /// Load and validate every `*.toml` file in `dir`, keyed by file stem.
    ///
    /// # Errors
    /// Returns an error if a file cannot be read, parsed, or fails validation.
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let mut items = BTreeMap::new();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let text = fs::read_to_string(&path)?;
            let quiz = toml::from_str::<Quiz>(&text)?.validate(id)?;
            items.insert(id.to_owned(), quiz);
        }
        Ok(Self { items })
    }

    /// Iterate over quizzes as `(id, quiz)` in id order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Quiz)> {
        self.items.iter()
    }

    /// Look up a quiz by id (the file stem).
    pub fn get(&self, id: &str) -> Option<&Quiz> {
        self.items.get(id)
    }
}

impl Quiz {
    /// Find a question by its id.
    pub fn question(&self, qid: u32) -> Option<&Question> {
        self.questions.iter().find(|q| q.id == qid)
    }
}

impl Question {
    /// Whether `selected` option indices (multi/single) or `text` (text kind)
    /// match this question's correct answer.
    pub fn is_correct(&self, kind: Kind, selected: &[usize], text: &str) -> bool {
        match kind {
            Kind::Multi => {
                let want = self
                    .options
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| o.correct)
                    .map(|(i, _)| i)
                    .collect::<HashSet<_>>();
                let got = selected.iter().copied().collect::<HashSet<_>>();
                want == got
            }
            Kind::Single => {
                selected.len() == 1 && self.options.get(selected[0]).is_some_and(|o| o.correct)
            }
            Kind::Text => {
                let want = text.trim().to_lowercase();
                !want.is_empty() && self.answers.iter().any(|a| a.trim().to_lowercase() == want)
            }
        }
    }
}

impl Quiz {
    /// Check that every question is well-formed for this quiz's `kind`.
    fn validate(self, id: &str) -> Result<Self, Error> {
        let mut seen = HashSet::new();
        for q in &self.questions {
            if !seen.insert(q.id) {
                return Err(Error::Content(format!(
                    "{id}: duplicate question id {}",
                    q.id
                )));
            }
            if q.prompt.trim().is_empty() {
                return Err(Error::Content(format!(
                    "{id}: question {} has an empty prompt",
                    q.id
                )));
            }
            match self.kind {
                Kind::Multi | Kind::Single => {
                    if q.options.iter().any(|o| o.text.trim().is_empty()) {
                        return Err(Error::Content(format!(
                            "{id}: question {} has an empty option",
                            q.id
                        )));
                    }
                    let correct = q.options.iter().filter(|o| o.correct).count();
                    if correct == 0 {
                        return Err(Error::Content(format!(
                            "{id}: question {} has no correct option",
                            q.id
                        )));
                    }
                    if matches!(self.kind, Kind::Single) && correct != 1 {
                        return Err(Error::Content(format!(
                            "{id}: single question {} must have exactly one correct option",
                            q.id
                        )));
                    }
                }
                Kind::Text => {
                    if q.answers.is_empty() {
                        return Err(Error::Content(format!(
                            "{id}: text question {} has no answers",
                            q.id
                        )));
                    }
                }
            }
        }
        Ok(self)
    }
}
