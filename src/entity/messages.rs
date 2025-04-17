use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub message: String,
    #[sea_orm(created_at)]
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SenderId",
        to = "super::users::Column::Id"
    )]
    Sender,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::ReceiverId",
        to = "super::users::Column::Id"
    )]
    Receiver,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sender.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
