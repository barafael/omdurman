# Handover: bevy_egui redraws every frame (upstream bug)

For an agent preparing an upstream fix (issue and PR) to
[bevy_egui](https://github.com/vladbat00/bevy_egui). Found while profiling
the omdurman game, 2026-10-03.

## Symptom

With a reactive `WinitSettings` (`UpdateMode::reactive` /
`reactive_low_power`), a Bevy app that uses bevy_egui never goes idle: it
renders at the display rate (60 fps here) with no input and nothing
animating. In omdurman an idle game-over screen used ~124% CPU, and still
~115% after switching to reactive mode.

Instrumentation (frames and `RequestRedraw` messages counted per 2 s, plus
`egui::Context::repaint_causes()`) showed, with no input at all:

```
frames=121 redraw_msgs=242 callers={} causes={"egui-0.36.2/src/context.rs:538", "EV ModifiersChanged"}
```

## Root cause: a self-sustaining loop

1. **bevy_egui pushes `ModifiersChanged` into every context's input on every
   frame**, changed or not. In `bevy_egui-0.42.0/src/input.rs`, at the end of
   `write_egui_input_system` (around line 1501):

   ```rust
   for (entity, mut egui_input, camera) in egui_contexts.iter_mut() {
       // ...
       egui_input.events.push(egui::Event::ModifiersChanged(
           modifier_keys_state.to_egui_modifiers(),
       ));
       egui_input.time = Some(time.elapsed_secs_f64());
   }
   ```

   `ModifierKeysState` is a resource that `write_modifiers_keys_state_system`
   (input.rs:376) updates from `KeyboardInput` and `KeyboardFocusLost`. The
   push happens whether it changed or not.

2. **egui repaints immediately on any input event.**
   `egui-0.36.2/src/input_state/mod.rs:655`, in `InputState::wants_repaint_after`:

   ```rust
   if self.pointer.wants_repaint()
       || self.wheel.unprocessed_wheel_delta.abs().max_elem() > 0.2
       || !self.events.is_empty()   // <- one ModifiersChanged is enough
       || ...
   {
       return Some(Duration::ZERO);   // immediate repaint
   }
   ```

   `Context::begin_pass` (context.rs:469 and :537-538) then calls
   `request_repaint_after(Duration::ZERO, ...)`. That is the
   `context.rs:538` repaint cause above.

3. **bevy_egui turns the repaint into a frame.**
   `bevy_egui-0.42.0/src/output.rs:111-116`, `process_output_system`:

   ```rust
   let needs_repaint = !render_output.is_empty();
   should_request_redraw |= ctx.has_requested_repaint() && needs_repaint;
   // ...
   if should_request_redraw {
       request_redraw_writer.write(RequestRedraw);
   }
   ```

   `RequestRedraw` wakes the reactive winit loop. The next frame pushes
   `ModifiersChanged` again, and so on. Two contexts gave the 2 messages per
   frame seen above.

So no update mode can make a bevy_egui app idle while any egui output is
drawn. That likely affects every bevy_egui user relying on
`WinitSettings::desktop_app()` or reactive modes.

## Workaround in omdurman (for reference)

`omdurman-app/src/activity.rs`, `drop_unchanged_modifiers`. It runs in
`PreUpdate`, `.after(EguiPreUpdateSet::ProcessInput).before(EguiPreUpdateSet::BeginPass)`,
and drops a `ModifiersChanged` event whose modifiers equal the last one seen
for that context:

```rust
pub fn drop_unchanged_modifiers(
    mut inputs: Query<(Entity, &mut bevy_egui::EguiInput)>,
    mut last: Local<HashMap<Entity, bevy_egui::egui::Modifiers>>,
) {
    use bevy_egui::egui::Event;
    for (entity, mut input) in &mut inputs {
        input.0.events.retain(|event| match event {
            Event::ModifiersChanged(now) => last.insert(entity, *now) != Some(*now),
            _ => true,
        });
    }
}
```

Result: idle CPU at game over went from ~115% to ~23% (with a 100 ms reactive
wait and a debug probe still running). `RawInput` (egui 0.36) has no
`modifiers` field, so the last value has to be tracked from the events.

## Suggested upstream fix

In `write_egui_input_system`, push `ModifiersChanged` only when the modifiers
differ from the last value pushed for that context. Options:

- Keep the last-sent `egui::Modifiers` per context, e.g. a field on
  `EguiInput`'s owner, a small component, or a `Local<HashMap<Entity, Modifiers>>`.
- Or set a "modifiers changed" flag in `write_modifiers_keys_state_system`
  (it already sees every relevant `KeyboardInput` and `KeyboardFocusLost`)
  and push only when it is set. Note that a newly added context should
  still get one initial `ModifiersChanged`.

Things to check while fixing:

- **Why the event exists.** egui keeps `InputState.modifiers` from
  `RawInput::modifiers`, and the per-frame event may exist to keep it in
  sync. In egui 0.36 the field is gone from `RawInput`, so modifiers reach
  egui only through `ModifiersChanged` (input_state/mod.rs:431 handles the
  event). Confirm that egui keeps the last modifiers between frames when no
  event arrives, so dropping unchanged events loses no state. omdurman's
  workaround relies on that, and Shift/Ctrl behaviour in egui widgets looked
  unaffected.
- **Other per-frame pushes.** Check the rest of `write_egui_input_system` and
  the input systems for events or fields written every frame that would
  also make `events` non-empty or `wants_repaint_after` return
  `Some(ZERO)`.
- **Focus loss.** `write_modifiers_keys_state_system` resets modifiers on
  `KeyboardFocusLost`; that change must still be sent.
- **Multiple contexts.** Track per context. Render-to-texture and secondary
  windows each have their own `EguiInput`.

## Reproduction for the issue

A minimal app: `DefaultPlugins`, `EguiPlugin`, one window, a system drawing
one `egui::Window` with a label, and
`WinitSettings { focused_mode: UpdateMode::reactive(Duration::from_secs(5)), unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(60)) }`.
Count frames per second (or `RequestRedraw` messages) with no input. With
0.42.0 it runs at vsync. After the fix it should tick about every 5 s.

Also check whether bevy_egui `main` (newer than 0.42.0) already fixed this
before filing; search the issues for "ModifiersChanged", "reactive",
"RequestRedraw" and "CPU usage".

## Versions

- bevy_egui 0.42.0 (`Cargo.toml`: `bevy_egui = "0.42"`), egui 0.36.2
- bevy 0.19.1 (bevy_winit 0.19.1), Rust 1.98.0
- Linux (CachyOS), KDE Plasma Wayland, game under XWayland
