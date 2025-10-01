pub mod context;
pub mod schema;

pub use context::{GraphQLContext, Joke};
pub use schema::QueryRoot;

use juniper::{EmptyMutation, EmptySubscription, RootNode};

pub type Schema =
    RootNode<QueryRoot, EmptyMutation<GraphQLContext>, EmptySubscription<GraphQLContext>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, EmptyMutation::new(), EmptySubscription::new())
}
