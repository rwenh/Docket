//! Diesel schema definitions for the task manager.
//!
//! Generated / maintained manually to stay in sync with the database migrations.

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        hashed_password -> Varchar,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    tasks (id) {
        id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        status -> Text,
        priority -> Text,
        due_date -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        owner_id -> Int4,
    }
}

diesel::joinable!(tasks -> users (owner_id));
diesel::allow_tables_to_appear_in_same_query!(users, tasks);
