use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "blocked_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,         // The user who is blocking
    pub blocked_user_id: Uuid, // The user who is being blocked

    #[sea_orm(created_at)]
    pub created_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Blocker,

    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::BlockedUserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Blocked,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Blocker.def()
    }

    fn via() -> Option<RelationDef> {
        None
    }
}

impl ActiveModelBehavior for ActiveModel {}
