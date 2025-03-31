use sea_orm::entity::prelude::*;

pub mod messages;
pub mod users;

pub use messages::Entity as Messages;
pub use users::Entity as Users;
