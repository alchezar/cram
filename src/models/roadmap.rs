//! Index roadmap: sections of B2 topics, some backed by a live quiz.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::Error;

/// The whole roadmap: page copy plus grouped topics.
#[derive(Debug, Deserialize)]
pub struct Roadmap {
    /// Page heading.
    pub title: String,
    /// Intro line under the heading.
    pub lead: String,
    /// Footer note.
    pub footer: String,
    /// Topic groups in display order.
    #[serde(rename = "section")]
    pub sections: Vec<Section>,
}

/// One roadmap section with its topics.
#[derive(Debug, Deserialize)]
pub struct Section {
    /// Section heading.
    pub name: String,
    /// Topics in this section, in display order.
    #[serde(default, rename = "topic")]
    pub topics: Vec<Topic>,
}

/// One roadmap topic. `quiz` links it to a live quiz id; absent means "Planned".
#[derive(Debug, Deserialize)]
pub struct Topic {
    /// Display number.
    pub n: u32,
    /// Topic title.
    pub title: String,
    /// Short description.
    pub desc: String,
    /// Backing quiz id, if the topic is live.
    #[serde(default)]
    pub quiz: Option<String>,
}

impl Roadmap {
    /// Load and parse the roadmap from a TOML file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, Error> {
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    /// Iterate over every topic across all sections.
    pub fn topics(&self) -> impl Iterator<Item = &Topic> {
        self.sections.iter().flat_map(|s| &s.topics)
    }
}
