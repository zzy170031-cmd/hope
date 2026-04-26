# export-engine

Excel-first export engine plus JSON and Markdown debug exports.

## Week3 export fixtures

Benchmark tests write generated week3 inputs, validation reports, and exports to temporary directories. They must not rewrite the tracked baseline files under `contracts/fixtures/exports`.

Updating the tracked week3 export baseline is explicit only:

```powershell
$env:HOPE_UPDATE_EXPORT_FIXTURES = '1'
cargo run -p export-engine --bin week3_export
```
