//! Generic undo/redo machinery, shared by every table editor.
//!
//! Each editor defines a concrete command enum implementing [`EditorCommand`]
//! (with inherent `apply`/`revert` methods over its own document type — the
//! history never needs to know them) and owns a [`History`]. Rapid edits to
//! the same target (dragging a value, typing) merge into one undo step via
//! [`EditorCommand::merge`].

use std::time::{Duration, Instant};

/// Edits to the same target within this window merge into one undo step.
const COALESCE_WINDOW: Duration = Duration::from_millis(750);

pub trait EditorCommand: Clone + std::fmt::Debug {
    /// Stable target identifier (`"cell/3/2"`); commands with the same key
    /// recorded within the coalesce window may merge.
    fn coalesce_key(&self) -> Option<String>;
    /// Fold `next` into `self` (both target the same edit). Returns whether
    /// the merge happened; the default refuses.
    fn merge(&mut self, next: &Self) -> bool {
        let _ = next;
        false
    }
}

pub struct History<C> {
    undo: Vec<C>,
    redo: Vec<C>,
    coalesce: Option<(String, Instant)>,
}

impl<C: EditorCommand> History<C> {
    pub fn new() -> Self {
        History { undo: Vec::new(), redo: Vec::new(), coalesce: None }
    }

    /// Record an already-applied command; callers apply/revert themselves.
    pub fn record(&mut self, cmd: C) {
        let ty = std::any::type_name::<C>();
        if let Some(key) = cmd.coalesce_key() {
            let still_hot = self
                .coalesce
                .as_ref()
                .is_some_and(|(k, at)| *k == key && at.elapsed() < COALESCE_WINDOW);
            if still_hot
                && self.undo.last_mut().is_some_and(|top| top.merge(&cmd))
            {
                log::debug!("history({ty}): merged into hot entry {key:?}");
                self.coalesce = Some((key, Instant::now()));
                self.redo.clear();
                return;
            }
            log::trace!("history({ty}): recorded {cmd:?}");
            self.coalesce = Some((key, Instant::now()));
        } else {
            log::trace!("history({ty}): recorded {cmd:?}");
            self.coalesce = None;
        }
        self.undo.push(cmd);
        self.redo.clear();
    }

    /// Pop the undo stack; the caller reverts the returned command.
    pub fn undo(&mut self) -> Option<C> {
        let cmd = self.undo.pop()?;
        log::trace!("history({}): undo {:?}", std::any::type_name::<C>(), cmd);
        self.redo.push(cmd.clone());
        self.coalesce = None;
        Some(cmd)
    }

    /// Pop the redo stack; the caller re-applies the returned command.
    pub fn redo(&mut self) -> Option<C> {
        let cmd = self.redo.pop()?;
        log::trace!("history({}): redo {:?}", std::any::type_name::<C>(), cmd);
        self.undo.push(cmd.clone());
        self.coalesce = None;
        Some(cmd)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

impl<C> Default for History<C> {
    fn default() -> Self {
        History { undo: Vec::new(), redo: Vec::new(), coalesce: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    enum Cmd {
        Set { key: String, old: u8, new: u8 },
        Add,
    }

    impl EditorCommand for Cmd {
        fn coalesce_key(&self) -> Option<String> {
            match self {
                Cmd::Set { key, .. } => Some(key.clone()),
                Cmd::Add => None,
            }
        }
        fn merge(&mut self, next: &Self) -> bool {
            match (self, next) {
                (Cmd::Set { new, .. }, Cmd::Set { new: next_new, .. }) => {
                    *new = *next_new;
                    true
                }
                _ => false,
            }
        }
    }

    #[test]
    fn coalescing_and_stacks() {
        let mut h = History::<Cmd>::new();
        h.record(Cmd::Set { key: "a".into(), old: 0, new: 1 });
        h.record(Cmd::Set { key: "a".into(), old: 1, new: 2 });
        h.record(Cmd::Set { key: "a".into(), old: 2, new: 3 });
        assert!(h.can_undo() && h.can_redo() == false);

        // Different target breaks the chain.
        h.record(Cmd::Set { key: "b".into(), old: 0, new: 9 });

        // Non-coalescing command resets.
        h.record(Cmd::Add);

        // Undo returns most recent first.
        match h.undo() {
            Some(Cmd::Add) => {}
            other => panic!("expected Add, got {other:?}"),
        }
        match h.undo() {
            Some(Cmd::Set { key, new, .. }) => {
                assert_eq!((key.as_str(), new), ("b", 9));
            }
            other => panic!("expected Set, got {other:?}"),
        }
        // The three `a` edits merged into one whose `new` is the latest.
        match h.undo() {
            Some(Cmd::Set { key, old, new }) => {
                assert_eq!((key.as_str(), old, new), ("a", 0, 3));
            }
            other => panic!("expected Set, got {other:?}"),
        }
        assert!(!h.can_undo());

        // Redo re-pushes.
        assert!(matches!(h.redo(), Some(Cmd::Set { .. })));
        assert!(h.can_undo());
    }
}
