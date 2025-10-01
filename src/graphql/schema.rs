use anyhow::{Error as AnyhowError, Result as AnyhowResult};
use juniper::{FieldResult, GraphQLObject, Value, graphql_object};

use crate::{db, pkgs::package::PackageDefinition};

use super::context::{GraphQLContext, Joke};

#[derive(Clone, GraphQLObject)]
#[graphql(description = "Package metadata exposed via the GraphQL API")]
pub struct PackageType {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub md5: Option<String>,
    pub configure_args: Vec<String>,
    pub build_commands: Vec<String>,
    pub install_commands: Vec<String>,
    pub dependencies: Vec<String>,
    pub enable_lto: bool,
    pub enable_pgo: bool,
    pub cflags: Vec<String>,
    pub ldflags: Vec<String>,
    pub profdata: Option<String>,
}

impl From<PackageDefinition> for PackageType {
    fn from(pkg: PackageDefinition) -> Self {
        let optimizations = pkg.optimizations;

        Self {
            name: pkg.name,
            version: pkg.version,
            source: pkg.source,
            md5: pkg.md5,
            configure_args: pkg.configure_args,
            build_commands: pkg.build_commands,
            install_commands: pkg.install_commands,
            dependencies: pkg.dependencies,
            enable_lto: optimizations.enable_lto,
            enable_pgo: optimizations.enable_pgo,
            cflags: optimizations.cflags,
            ldflags: optimizations.ldflags,
            profdata: optimizations.profdata,
        }
    }
}

#[derive(Clone, GraphQLObject)]
#[graphql(description = "A light-hearted package-related joke")]
pub struct JokeType {
    pub id: String,
    pub package: Option<String>,
    pub text: String,
}

impl From<Joke> for JokeType {
    fn from(joke: Joke) -> Self {
        Self {
            id: joke.id,
            package: joke.package,
            text: joke.text,
        }
    }
}

#[derive(Default)]
pub struct QueryRoot;

#[graphql_object(context = GraphQLContext)]
impl QueryRoot {
    fn packages(context: &GraphQLContext, limit: Option<i32>) -> FieldResult<Vec<PackageType>> {
        let limit = limit.unwrap_or(50).clamp(1, 200) as usize;
        let definitions =
            db::load_package_definitions_via_pool(&context.db_pool).map_err(field_error)?;

        Ok(definitions
            .into_iter()
            .take(limit)
            .map(PackageType::from)
            .collect())
    }

    fn package(
        context: &GraphQLContext,
        name: String,
        version: Option<String>,
    ) -> FieldResult<Option<PackageType>> {
        let definition =
            db::find_package_definition_via_pool(&context.db_pool, &name, version.as_deref())
                .map_err(field_error)?;

        Ok(definition.map(PackageType::from))
    }

    fn search(
        context: &GraphQLContext,
        query: String,
        limit: Option<i32>,
    ) -> FieldResult<Vec<PackageType>> {
        let limit = limit.map(|value| i64::from(value.clamp(1, 200)));
        let results =
            db::search_packages_via_pool(&context.db_pool, &query, limit).map_err(field_error)?;

        let packages = results
            .into_iter()
            .map(|pkg| pkg.into_definition().map(PackageType::from))
            .collect::<AnyhowResult<Vec<_>>>()
            .map_err(field_error)?;

        Ok(packages)
    }

    fn jokes(context: &GraphQLContext, package: Option<String>) -> FieldResult<Vec<JokeType>> {
        let catalog = context.joke_catalog();
        Ok(catalog
            .all(package.as_deref())
            .into_iter()
            .map(JokeType::from)
            .collect())
    }

    fn random_joke(
        context: &GraphQLContext,
        package: Option<String>,
    ) -> FieldResult<Option<JokeType>> {
        let catalog = context.joke_catalog();
        Ok(catalog.random(package.as_deref()).map(JokeType::from))
    }
}

fn field_error(err: AnyhowError) -> juniper::FieldError {
    juniper::FieldError::new(err.to_string(), Value::null())
}
