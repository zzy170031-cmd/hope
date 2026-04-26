fn main() -> Result<(), String> {
    if std::env::var("HOPE_UPDATE_EXPORT_FIXTURES").as_deref() != Ok("1") {
        return Err(
            "refusing to update tracked week3 export fixtures; set HOPE_UPDATE_EXPORT_FIXTURES=1 to opt in"
                .to_string(),
        );
    }

    export_engine::export_week3_from_fixtures()
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}
