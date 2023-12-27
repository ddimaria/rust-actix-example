// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 100]
        first_name -> Varchar,
        #[max_length = 100]
        last_name -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 122]
        password -> Varchar,
        #[max_length = 36]
        created_by -> Varchar,
        created_at -> Timestamp,
        #[max_length = 36]
        updated_by -> Varchar,
        updated_at -> Timestamp,
    }
}
