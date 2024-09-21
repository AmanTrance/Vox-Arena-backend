// @generated automatically by Diesel CLI.

diesel::table! {
    sol_balance (user_id) {
        balance -> Float8,
        user_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        email -> Text,
        password -> Text,
        token -> Text,
    }
}

diesel::joinable!(sol_balance -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sol_balance,
    users,
);
