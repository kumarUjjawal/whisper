use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(created_at)]
    pub created_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::messages::Entity",
        from = "Column::Id",
        to = "super::messages::Column::SenderId"
    )]
    SentMessages,
    #[sea_orm(
        has_many = "super::messages::Entity",
        from = "Column::Id",
        to = "super::messages::Column::ReceiverId"
    )]
    ReceivedMessages,
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SentMessages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
