# HiveMind Roadmap — UI/UX & HoneyGlass

This is the dedicated roadmap section for the FigCraft-derived interface. The
detailed material/component contract lives in
[`ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md`](ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md).

## UI/UX-0 — preserve

- [x] Keep FigCraft tokens, information architecture, section order, and every
      existing feature.
- [x] Keep Rust as the runtime and React/TypeScript as the presentation layer.
- [x] Keep the 10–20% change ceiling for visual/UX polish.

## UI/UX-1 — startup veil (implemented)

- [x] Add `HiveMindSplash` with a live connection gate.
- [x] HoneyGlass visual: frosted pane + liquid honeycomb + neumorphic progress.
- [x] Wordmark and description with the **Designed by Riley Christensen**
      glass credit in the bottom-right.
- [x] Finite motion, reduced-motion support, opaque `backdrop-filter` fallback.
- [x] Capture a connected 1440×900 product screenshot.
- [ ] Capture 375px and keyboard/focus evidence.

### Pseudocode / real code

```text
PSEUDOCODE                              REAL CODE
veil visible until settled             const settled = connection === 'open' || ...
minimum 1600ms                         setTimeout(... setMinimumElapsed(true), 1_600)
bounded wait 4400ms                    setTimeout(... setPhase('leaving'), 4_400)
fade 420ms then unmount                setTimeout(... setPhase('gone'), 420)
```

```text
PSEUDOCODE                              REAL CODE
glass pane                             background: linear-gradient(...) + backdrop-filter
liquid focal hex                       clip-path: polygon(...) + honey gradient
neumorphic loading well                box-shadow inset + raised gradient fill
```

## UI/UX-2 — material tokens

- [ ] Promote glass/neumorphic values into the token mirror.
- [ ] Add automated contrast checks for all glass text pairs.
- [ ] Add a visual-regression harness for 1440×900 and 375px.

## UI/UX-3 — transient overlays

- [ ] Apply HoneyGlass to approval + reconnect overlays only.
- [ ] Keep the persistent canvas/navigation opaque and quiet.
- [ ] Add a single-specular rule and test it.

## UI/UX-4 — motion and performance

- [ ] Profile graph animation at 40 nodes.
- [ ] Keep transform/opacity-only animation paths.
- [ ] Capture a reduced-motion screenshot variant.

## UI/UX-5 — system completion

- [ ] Publish the HoneyGlass system as the canonical interface contract.
- [ ] Keep FigCraft source assets read-only and provenance-labelled.
- [ ] Review every future brand change against token, contrast, and scope gates.

## Status boundaries

The implementation added the startup veil and the measured polish pass. It did
**not** replace FigCraft, rewrite the app shell, or add a new palette. The
unchecked items remain deliberate roadmap work, not hidden completion claims.
