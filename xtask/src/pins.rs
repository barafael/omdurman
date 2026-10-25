//! The box builds the signalling server from the matchbox fork the game depends on, but names that
//! revision in a file of its own: testbox/matchbox/Containerfile's argument. A test fails when it
//! drifts from the revision Cargo.lock resolves.

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// The 40-hex-digit revisions that follow `marker` in `text`, in order, without repeats.
    fn revisions_after(text: &str, marker: &str) -> Vec<String> {
        let mut found: Vec<String> = text
            .match_indices(marker)
            .filter_map(|(at, _)| {
                let rest = &text[at + marker.len()..];
                let revision: String = rest.chars().take_while(char::is_ascii_hexdigit).collect();
                (revision.len() == 40).then_some(revision)
            })
            .collect();
        found.dedup();
        found
    }

    fn read(repo: &Path, path: &str) -> String {
        std::fs::read_to_string(repo.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    }

    #[test]
    fn the_box_serves_on_the_games_matchbox() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let locked = revisions_after(
            &read(repo, "Cargo.lock"),
            "git+https://github.com/barafael/matchbox.git?branch=main#",
        );
        assert_eq!(locked.len(), 1, "Cargo.lock's matchbox: {locked:?}");
        let built = revisions_after(
            &read(repo, "testbox/matchbox/Containerfile"),
            "ARG MATCHBOX_REV=",
        );
        assert_eq!(built, locked, "testbox/matchbox/Containerfile");
    }
}
