use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub phone_number: String,
    // pub phone_hash: String,
    pub name: String,
    pub photo_url: Option<String>,
    pub status_message: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTimeUtc>,

    #[sea_orm(created_at)]
    pub created_at: Option<DateTimeUtc>,
    pub updated_at: Option<DateTimeUtc>,
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

    #[sea_orm(
        has_many = "super::blocked_users::Entity",
        from = "Column::Id",
        to = "super::blocked_users::Column::UserId"
    )]
    BlockedUsers,

    #[sea_orm(
        has_many = "super::blocked_users::Entity",
        from = "Column::Id",
        to = "super::blocked_users::Column::BlockedUserId"
    )]
    BlockedBy,
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SentMessages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
