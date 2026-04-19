fn main() -> Result<(), String> {
    export_engine::export_week3_from_fixtures()
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}
