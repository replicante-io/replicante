extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate serde_json;

extern crate replicante_agent_models;


mod node;

pub use self::node::Node;
