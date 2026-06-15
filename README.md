## Traceability

Validate the bijective rulebook-to-code mapping:

    cargo test -p omdurman-rules --test traceability

Regenerate the PDF report (requires `typst` CLI):

    cargo run -p traceability-typst
