#![cfg(feature = "graphql")]

use std::env;
use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware::Compress, web};
use anyhow::{Context, Result};
use juniper_actix::{graphiql_handler, graphql_handler};

use package_management::db;
use package_management::graphql::{self, GraphQLContext, Schema};

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(err) = run().await {
        eprintln!("GraphQL server failed: {err:#}");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ));
    }

    Ok(())
}

async fn run() -> Result<()> {
    let pool = db::establish_pool().context("initialising SQLite pool")?;
    let schema = Arc::new(graphql::create_schema());
    let jokes = Arc::new(graphql::context::JokeCatalog::default());
    let bind_addr = env::var("LPKG_GRAPHQL_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    let workers = worker_count();

    println!("GraphQL server listening on {bind_addr} with {workers} worker(s)");

    HttpServer::new(move || {
        let app_schema = Arc::clone(&schema);
        let pool = pool.clone();
        let jokes = Arc::clone(&jokes);

        App::new()
            .app_data(web::Data::from(app_schema))
            .app_data(web::Data::new(pool))
            .app_data(web::Data::from(jokes))
            .wrap(Compress::default())
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_endpoint))
                    .route(web::get().to(graphql_endpoint)),
            )
            .service(web::resource("/playground").route(web::get().to(graphiql_endpoint)))
    })
    .workers(workers)
    .bind(&bind_addr)
    .with_context(|| format!("binding GraphQL server to {bind_addr}"))?
    .run()
    .await
    .context("running GraphQL server")
}

async fn graphql_endpoint(
    schema: web::Data<Arc<Schema>>,
    pool: web::Data<db::Pool>,
    jokes: web::Data<Arc<graphql::context::JokeCatalog>>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let context = GraphQLContext::with_catalog(pool.get_ref().clone(), Arc::clone(jokes.get_ref()));
    graphql_handler(schema.get_ref().as_ref(), &context, req, payload).await
}

async fn graphiql_endpoint() -> Result<HttpResponse, actix_web::Error> {
    graphiql_handler("/graphql", None).await
}

fn worker_count() -> usize {
    let suggested = num_cpus::get();
    suggested.clamp(1, 8)
}
