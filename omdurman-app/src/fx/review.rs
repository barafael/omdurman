//! The review timeline's marks: the event under the cursor shows its fire
//! traces or its melee mark, the same marks live play raises (held while the
//! timeline is paused, see [`super::transient::animate_transients`]).

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use omdurman_net::GameEvent;
use omdurman_rules::effects::GameEffect;

use super::FxRequest;
use super::transient::Transient;
use crate::timeline::SpectatorTimeline;

/// Once per (record, cursor): drop the marks of the event the cursor left,
/// and mark the event it stands on --
///
/// - `FireCombat` / `HowitzerFire`: a trace from each firing hex to the
///   target (§6); a howitzer's (§6.64) at the *aimed* hex -- where the shell
///   scattered shows on the board;
/// - `DeclareMelee`: a mark between the warring hexes, pointing at the
///   defenders (§7).
///
/// Firer positions come from the rebuilt state, "after this event", i.e.
/// where the firers stood when they fired (fire moves no unit).
pub(super) fn review_marks(
    timeline: Res<SpectatorTimeline>,
    game_state: Res<crate::GameStateResource>,
    marks: Query<Entity, With<Transient>>,
    mut fx: MessageWriter<FxRequest>,
    mut commands: Commands,
    // (record label, cursor, generation) of the last event marked, so a
    // re-scrub of the same event doesn't restart its marks; the generation
    // counts re-opening the same record parked on the same event.
    mut last: Local<Option<(String, usize, u32)>>,
) {
    let Some(record) = timeline.record.as_ref() else {
        *last = None;
        return;
    };
    let key = (
        timeline.source_label.clone(),
        timeline.cursor,
        timeline.generation,
    );
    if last.as_ref() == Some(&key) {
        return;
    }
    *last = Some(key);
    for entity in &marks {
        commands.entity(entity).despawn();
    }
    let Some(GameEvent::Effect(effect)) = record.events.get(timeline.cursor).map(|e| &e.payload)
    else {
        return;
    };
    match effect {
        GameEffect::FireCombat { attack, .. } | GameEffect::HowitzerFire { attack, .. } => {
            let firers = attack.all_firing_units();
            let from_hexes = super::announce::firing_hexes(&game_state.0, &firers);
            if from_hexes.is_empty() {
                // Every firer is gone from the rebuilt state (an invalid
                // record whose earlier events no longer replay cleanly).
                debug!(
                    cursor = timeline.cursor,
                    ?attack,
                    "review: fire marks skipped, no firer resolved"
                );
            }
            for from in from_hexes {
                fx.write(FxRequest::Shot {
                    from,
                    to: attack.target_hex,
                    scatter: false,
                });
            }
        }
        GameEffect::DeclareMelee { attack, .. } => {
            fx.write(FxRequest::Clash {
                attacker: attack.attacker_hex,
                defender: attack.defender_hex,
            });
        }
        _ => {}
    }
}
