use std::sync::Arc;

use rand::rng;
use rand::seq::IteratorRandom;

use crate::db;

#[derive(Clone)]
pub struct GraphQLContext {
    pub db_pool: db::Pool,
    jokes: Arc<JokeCatalog>,
}

impl GraphQLContext {
    pub fn new(db_pool: db::Pool) -> Self {
        Self {
            db_pool,
            jokes: Arc::new(JokeCatalog::default()),
        }
    }

    pub fn with_jokes(db_pool: db::Pool, jokes: Vec<Joke>) -> Self {
        Self {
            db_pool,
            jokes: Arc::new(JokeCatalog::new(jokes)),
        }
    }

    pub fn with_catalog(db_pool: db::Pool, catalog: Arc<JokeCatalog>) -> Self {
        Self {
            db_pool,
            jokes: catalog,
        }
    }

    pub fn joke_catalog(&self) -> Arc<JokeCatalog> {
        Arc::clone(&self.jokes)
    }
}

impl juniper::Context for GraphQLContext {}

#[derive(Clone, Debug)]
pub struct Joke {
    pub id: String,
    pub package: Option<String>,
    pub text: String,
}

impl Joke {
    pub fn new(id: impl Into<String>, package: Option<&str>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            package: package.map(|pkg| pkg.to_string()),
            text: text.into(),
        }
    }
}

#[derive(Clone)]
pub struct JokeCatalog {
    entries: Arc<Vec<Joke>>,
}

impl JokeCatalog {
    fn new(entries: Vec<Joke>) -> Self {
        Self {
            entries: Arc::new(entries),
        }
    }

    pub fn random(&self, package: Option<&str>) -> Option<Joke> {
        let mut rng = rng();

        if let Some(package) = package {
            if let Some(chosen) = self
                .entries
                .iter()
                .filter(|joke| matches_package(joke, package))
                .choose(&mut rng)
            {
                return Some(chosen.clone());
            }
        }

        self.entries.iter().choose(&mut rng).cloned()
    }

    pub fn all(&self, package: Option<&str>) -> Vec<Joke> {
        match package {
            Some(package) => self
                .entries
                .iter()
                .filter(|joke| matches_package(joke, package))
                .cloned()
                .collect(),
            None => self.entries.as_ref().clone(),
        }
    }
}

impl Default for JokeCatalog {
    fn default() -> Self {
        Self::new(default_jokes())
    }
}

fn matches_package(joke: &Joke, package: &str) -> bool {
    joke.package
        .as_deref()
        .map(|pkg| pkg.eq_ignore_ascii_case(package))
        .unwrap_or(false)
}

fn default_jokes() -> Vec<Joke> {
    vec![
        Joke::new(
            "optimizer-overdrive",
            Some("gcc"),
            "The GCC optimizer walked into a bar, reordered everyone’s drinks, and they still tasted the same—just faster.",
        ),
        Joke::new(
            "linker-chuckle",
            Some("binutils"),
            "Our linker refuses to go on vacation; it can’t handle unresolved references to the beach.",
        ),
        Joke::new(
            "glibc-giggle",
            Some("glibc"),
            "The C library tried stand-up comedy but segfaulted halfway through the punchline.",
        ),
        Joke::new(
            "pkg-general",
            None,
            "LPKG packages never get lost—they always follow the dependency graph back home.",
        ),
    ]
}
