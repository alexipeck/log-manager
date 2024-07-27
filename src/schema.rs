// @generated automatically by Diesel CLI.

diesel::table! {
    log (id) {
        id -> Integer,
        source -> Text,
        timestamp -> Text,
        level -> Text,
        location -> Text,
        content -> Text,
    }
}
