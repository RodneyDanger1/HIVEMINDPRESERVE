# HiveMind HoneyGlass — Interface & UX Design System

Status: implemented foundation (startup veil + 10–20% polish pass), roadmap below.
Provenance: FigCraft output and the existing HiveMind token system. This document
does not replace FigCraft; it names the material rules that let the existing
design move into a more modern, paid-product finish without changing its identity.

## 1. The design thesis

HiveMind is a local HMARS, not a generic chat dashboard. The interface is an
apiary control room: paper and ink for the chrome, honey for the living system,
hex cells for the hive geometry. The visual idea is:

> One bright hive, quiet paper, glass only where the runtime is actually alive.

The UI should feel calm at rest and precise while working. Motion is never
decoration on top of missing information; it shows a real transition.

## 2. Foundations inherited from FigCraft

| Token family | Role | Source |
|---|---|---|
| `canvas`, `panel`, `panel-alt`, `recessed`, `hairline` | paper chrome | `web/src/styles/tokens.css` / `web/src/tokens/tokens.ts` |
| `honey-100..800` | living hive / focal moments | same |
| `ink-900..500` | three text roles | same |
| `signal`, `verify`, `amber` | named state | same |
| Playfair Display + Inter + mono | display / body / machine | `@fontsource` bundles |

No new palette was introduced by this system. Glass is a material over the
existing palette, not a second brand.

## 3. HoneyGlass material layers

### Glassmorphism — the pane

Used for the startup veil and, later, only transient overlays. A glass pane is:

```text
1px translucent white border
+ translucent white→honey gradient
+ backdrop blur(30px) saturate(1.18)
+ a specular highlight (screen blend)
+ a soft warm-ink shadow
```

It is never used for the persistent canvas or the main navigation. Those stay
opaque enough to preserve WCAG contrast and long-session legibility.

### Liquid Glass — the living focal object

The honeycomb cluster and the dripping wordmark are the one bright element.
The effect comes from layered gradients, an inner top highlight, a warm drop
shadow, and honey-drop geometry. It is not a neon glow.

### Neumorphism — the loading well

Only small loading/status controls use neumorphism: an inset recessed track and
a raised honey fill. Persistent buttons stay flat and obvious, because
low-contrast neumorphic controls are a usability trap.

### Google/OpenAI system lessons, used selectively

- strong type hierarchy and generous breathing room;
- one primary action, quiet secondary chrome;
- motion that confirms state;
- explicit empty/loading/error states;
- no invented content or impossible metrics.

We do not copy their layouts or brand surfaces.

## 4. Startup veil — "the hive waking up"

The startup surface is a real boot state, not a fake loading screen:

```text
on mount:
  start a 1.6s minimum visual window
  observe live connection state

if minimum elapsed and connection is settled:
  fade the veil
else if minimum elapsed and still connecting:
  keep the veil until settled, bounded at 4.4s
else:
  after the bounded wait, open the workspace and let the offline banner speak

status:
  connecting -> "Bringing the hive online"
  open       -> "The hive is awake"
  error      -> "Runtime is offline"
  reconnecting -> "Rejoining the hive"
```

The veil contains: HiveMind wordmark (Playfair, honey gradient, drip SVG),
one-sentence HMARS description, a neumorphic progress well, live status text,
and a bottom-right glass credit:

> Designed by **Riley Christensen**

Code: `web/src/splash/HiveMindSplash.tsx`, `web/src/styles/splash.css`.

## 5. Pseudocode / real code — lifecycle

```text
PSEUDOCODE                              REAL CODE
minimumElapsed = false                  const [minimumElapsed, setMinimumElapsed] = useState(false)
after 1600ms -> true                    window.setTimeout(() => setMinimumElapsed(true), 1_600)
settled = open || error || closed       const settled = connection === 'open' || ...
if minimumElapsed:                      useEffect(() => { if (!minimumElapsed) return;
  if settled -> leave                   ... setPhase('leaving') ... }, [...])
  else wait 4400ms -> leave
leave for 420ms -> unmount              setTimeout(() => setPhase('gone'), 420)
```

The veil cannot cover the app forever. If the runtime is down, it opens the
offline workspace after the bounded wait, and the real named refusal remains
visible.

## 6. Pseudocode / real code — material

```text
PSEUDOCODE                              REAL CODE
glass = translucent white/honey         background:
        + blur + saturate                linear-gradient(135deg, rgb(255 255 255 / .62), ...)
        + 1px white border               border: 1px solid rgb(255 255 255 / .76)
        + specular screen layer          .splash-glass::before { mix-blend-mode: screen; }
        + warm shadow                    box-shadow: 0 30px 80px rgb(40 32 16 / .2);
```

```text
PSEUDOCODE                              REAL CODE
hex = honey gradient                    .splash-hex { background: linear-gradient(...); }
    + clip-path hexagon                 clip-path: polygon(50% 0%, 93% 25%, ...)
    + finite entrance                   animation: splash-cell-in 900ms ... both;
```

```text
PSEUDOCODE                              REAL CODE
well = recessed inset track             box-shadow: 0 2px 5px rgb(40 32 16 / .18) inset
     + raised honey fill                .splash-fill { background: linear-gradient(90deg, ...) }
```

## 7. Motion rules

- One entrance curve: `cubic-bezier(0.22, 0.61, 0.36, 1)`.
- One progress transition: 520ms.
- One veil fade: 360ms.
- No infinite loops in the splash. The existing FigCraft ambient loops remain
  in the workspace and are governed by the ambient-motion preference.
- All motion is disabled by the existing `prefers-reduced-motion` /
  `data-reduced-motion` rule.

## 8. Component map

| Surface | Component | Material |
|---|---|---|
| Startup veil | `HiveMindSplash` | glass + liquid hex + neumorphic well |
| Header / nav / canvas | existing FigCraft components | opaque paper + honey accents |
| Status / error | existing banners | tinted neutral, signal text |
| Provider / settings | existing panels | flat panel, one honey action |
| Future transient palettes | not yet implemented | glass |

## 9. Acceptance rules

- Information architecture, section order, and features unchanged.
- No new palette outside the FigCraft token mirror.
- Every glass surface has an opaque fallback (`@supports not backdrop-filter`).
- Text contrast on the veil remains AA; status uses named text roles.
- Motion is finite on the splash and respects reduced motion.
- 1440×900 and ≤720px both render with the credit legible.
- Static audit cannot regress the 10–20% polish baseline: spacing U006 = 0,
  weight U003 = 0.

## 10. Roadmap — own section

### Phase UI-1 — startup veil (implemented this round)

- [x] HoneyGlass veil component and finite entrance.
- [x] Live connection/status gate with bounded fallback.
- [x] FigCraft wordmark, HMARS description, glass designer credit.
- [x] Opaque fallback for browsers without backdrop-filter.
- [x] 1440×900 render and connected-state screenshot.
- [ ] 375px screenshot and keyboard/focus evidence.

### Phase UI-2 — material tokens

- Add named `--material-glass-*` and `--material-neumorphic-*` variables to the
  token mirror so future glass surfaces cannot drift into ad-hoc values.
- Add contrast tests for every glass foreground/background pair.

### Phase UI-3 — persistent surfaces

- Apply HoneyGlass only to transient overlays (approval, command palette,
  reconnect state), never the canvas or main navigation.
- Add one specular highlight per glass surface, not per child.

### Phase UI-4 — motion/performance

- Profile startup and the 40-node graph; keep animations on transform/opacity.
- Add a reduced-motion screenshot variant.
- Add a visual regression render at 1440×900 and 375px.

### Phase UI-5 — design-system handoff

- Publish this document as the canonical HiveMind interface system.
- Keep FigCraft assets as read-only provenance.
- Any future brand experiment must pass the token, contrast, and 10–20% scope
  checks before replacing a surface.

## 11. Guardrails

- Do not turn the app into a glass dashboard. Glass is a startup/transient
  material, not the chrome.
- Do not use neumorphism for primary controls.
- Do not add decorative infinite motion to the startup veil.
- Do not change FigCraft's section order or remove features in the name of
  polish.
- Any new visual element must make the HMARS workflow clearer; if it only
  decorates, remove it.
