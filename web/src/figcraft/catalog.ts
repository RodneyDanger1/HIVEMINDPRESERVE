/**
 * GENERATED FigCraft catalog - do not hand-edit.
 *
 * Generator: tools/build-figcraft-catalog.mjs
 * Source   : docs/ui/figcraft/ASSETS-INDEX.json
 * Index sha: 1C2426CCC208B149465D5FDFCCACDB7E57A3F2AFC2E729EAF4767C551648BFC0
 * Index at : 2026-11-13T17:24:24+00:00
 * Served   : web/public/figcraft/<svgFile>; provenance in SOURCES.json
 *
 * Per-board audit status is part of the data, not a UI guess:
 *  - watchdog-audited: PASS/PASS-WITH-NOTES artifact in ledgers/ui-watchdog.json
 *  - self-verified   : HiveMind CLI verify report PASS; NOT a watchdog audit
 *  - proposed        : PROPOSED-ADDITION / motion proposal; not watchdog-audited
 *  - unverified      : no accepted audit evidence
 */
import type {
  FigCraftBoard,
  FigCraftGroup,
  FigCraftSourceManifest,
} from './types';
export const FIGCRAFT_GROUPS: readonly FigCraftGroup[] = [
  {
    "id": "figcraft-hivemind",
    "label": "HiveMind · self-verified / proposed",
    "description": "HiveMind spatial style system: its own CLI verify report passed, but it is not in the A/B watchdog ledger. Boards marked PROPOSED-ADDITION or motion proposal are labelled proposed."
  },
  {
    "id": "figcraft-a",
    "label": "Design system · A · watchdog-audited",
    "description": "Watchdog-audited design-system manifest A (foundations, brand, components, specimens). The ledger verdict covers this set per artifact."
  },
  {
    "id": "figcraft-b",
    "label": "Canvas · B · watchdog-audited",
    "description": "Watchdog-audited canvas/interaction manifest B (canvas default, zoom states, interaction, hero). The ledger verdict covers this set per artifact."
  }
];

export const FIGCRAFT_BOARDS: readonly FigCraftBoard[] = [
  {
    "id": "cover",
    "owner": "figcraft-hivemind",
    "title": "00 · Cover",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-cover.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-cover.svg",
    "sha256": "ad9f04ede5c800196e72a37f8496088190bcf840f07b2029fc4ea2fa5746ed62",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": []
  },
  {
    "id": "colour",
    "owner": "figcraft-hivemind",
    "title": "01 · Colour",
    "width": 1440,
    "height": 1240,
    "svgFile": "hivemind-colour.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-colour.svg",
    "sha256": "8da62cceb2d46f5c76abd3a4ffed2baf2957ef72aa3f7bccf9cdf36077e35360",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": []
  },
  {
    "id": "typography",
    "owner": "figcraft-hivemind",
    "title": "02 · Typography",
    "width": 1440,
    "height": 1340,
    "svgFile": "hivemind-typography.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-typography.svg",
    "sha256": "0236956e8f29ace3ea587ef0675c00701a1b5997502a6744220b4b95a1c77594",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": []
  },
  {
    "id": "space-and-depth",
    "owner": "figcraft-hivemind",
    "title": "03 · Space & Depth",
    "width": 1440,
    "height": 1000,
    "svgFile": "hivemind-space-and-depth.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-space-and-depth.svg",
    "sha256": "005871e538619453dd453955d221359ac89e54850e9ee5a36af836f4c4302378",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": []
  },
  {
    "id": "brand-lockups",
    "owner": "figcraft-hivemind",
    "title": "04 · Brand Lockups",
    "width": 1440,
    "height": 1000,
    "svgFile": "hivemind-brand-lockups.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-brand-lockups.svg",
    "sha256": "b83c5ca38c981e47b25a49a2e523c276c34d506c62fa84944a3aa865a758d473",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": [
      {
        "surfaceId": "S23",
        "name": "Brand lockups / bee glyph"
      }
    ]
  },
  {
    "id": "agent-cards",
    "owner": "figcraft-hivemind",
    "title": "05 · Agent Cards",
    "width": 1440,
    "height": 1300,
    "svgFile": "hivemind-agent-cards.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-agent-cards.svg",
    "sha256": "c021a836c169125aa4be37288a2fa72f9ddc4575c1ce01c2a10443d80329fd01",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": [
      {
        "surfaceId": "S15",
        "name": "SuperAgent card (420x150)"
      },
      {
        "surfaceId": "S16",
        "name": "WorkerAgent card (320x96)"
      },
      {
        "surfaceId": "S17",
        "name": "ToolAgent card (420x240)"
      },
      {
        "surfaceId": "S18",
        "name": "Broken-route error card (crimson)"
      },
      {
        "surfaceId": "S19",
        "name": "Agent chips / state dot"
      },
      {
        "surfaceId": "S24",
        "name": "Elevation e1/e2/e3"
      }
    ]
  },
  {
    "id": "controls-and-hud",
    "owner": "figcraft-hivemind",
    "title": "06 · Controls & HUD",
    "width": 1440,
    "height": 1040,
    "svgFile": "hivemind-controls-and-hud.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-controls-and-hud.svg",
    "sha256": "d8f92f5c66cfd9353b912013eb4ff8fc6039972c377f9673077805847057ea0d",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": [
      {
        "surfaceId": "S03",
        "name": "Composer / HyperAgent input"
      },
      {
        "surfaceId": "S04",
        "name": "Button: Spawn sub-hive"
      },
      {
        "surfaceId": "S05",
        "name": "Button: Re-route"
      },
      {
        "surfaceId": "S06",
        "name": "Button: Cancel"
      },
      {
        "surfaceId": "S07",
        "name": "Button: Kill worker"
      },
      {
        "surfaceId": "S08",
        "name": "Tabs: Workspace / Swarm / Routing / History"
      },
      {
        "surfaceId": "S10",
        "name": "Time-travel scrubber"
      }
    ]
  },
  {
    "id": "spatial-canvas",
    "owner": "figcraft-hivemind",
    "title": "07 · Spatial Canvas",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-spatial-canvas.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-spatial-canvas.svg",
    "sha256": "92256071aea74b869ea2145668e83af8cbd58e4bb130c321117d569ffed8bc4f",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "self-verified",
      "verdict": "PASS",
      "ledger": "docs/ui/figcraft/HiveMind/build/verify-report.json",
      "basis": "HiveMind verify_hivemind.py report PASS; not a watchdog audit"
    },
    "surfaces": [
      {
        "surfaceId": "S01",
        "name": "Infinite canvas + honeycomb background"
      },
      {
        "surfaceId": "S02",
        "name": "HyperAgent hive chat core (pinned)"
      },
      {
        "surfaceId": "S09",
        "name": "HUD throughput badge (460x44)"
      },
      {
        "surfaceId": "S11",
        "name": "Nav rail (72px, 6 items)"
      },
      {
        "surfaceId": "S12",
        "name": "Library panel (240px authoritative)"
      },
      {
        "surfaceId": "S13",
        "name": "Mini-map (220x150 authoritative)"
      },
      {
        "surfaceId": "S20",
        "name": "Zoom controls + shortcut hint"
      },
      {
        "surfaceId": "S27",
        "name": "Streamed reply / chat cards"
      },
      {
        "surfaceId": "S28",
        "name": "Project identity"
      }
    ]
  },
  {
    "id": "hmas-hierarchy",
    "owner": "figcraft-hivemind",
    "title": "08 · HMAS Hierarchy",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-hmas-hierarchy.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-hmas-hierarchy.svg",
    "sha256": "eb1ce756e061066a77304ff5c89860cf962e443f8f3c61e2e588da84a73a877d",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "proposed",
      "verdict": null,
      "ledger": null,
      "basis": "PROPOSED-ADDITION / motion proposal per docs/ui/figcraft/HiveMind/README.md; not watchdog-audited"
    },
    "surfaces": [
      {
        "surfaceId": "S22",
        "name": "File/code hierarchy viewer"
      },
      {
        "surfaceId": "HMAS-HIERARCHY",
        "name": "Hierarchy / workspace-jailed tree"
      },
      {
        "surfaceId": "HMAS-TOPOLOGY",
        "name": "Topology / live bee edges"
      }
    ]
  },
  {
    "id": "hmas-governance",
    "owner": "figcraft-hivemind",
    "title": "09 · HMAS Governance",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-hmas-governance.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-hmas-governance.svg",
    "sha256": "7c763f8f36758318502d084baffe82dcd5483246a22c412b19edf0771c264848",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "proposed",
      "verdict": null,
      "ledger": null,
      "basis": "PROPOSED-ADDITION / motion proposal per docs/ui/figcraft/HiveMind/README.md; not watchdog-audited"
    },
    "surfaces": [
      {
        "surfaceId": "HMAS-PROFILES",
        "name": "Profiles panel"
      },
      {
        "surfaceId": "HMAS-IMPORT",
        "name": "Import project"
      },
      {
        "surfaceId": "HMAS-MEMORY",
        "name": "Memory list + clear"
      },
      {
        "surfaceId": "HMAS-MAILBOX",
        "name": "Mailbox"
      },
      {
        "surfaceId": "HMAS-PROJECTS",
        "name": "Projects"
      },
      {
        "surfaceId": "HMAS-USAGE",
        "name": "Usage counters"
      },
      {
        "surfaceId": "HMAS-MESSAGE-APPROVAL",
        "name": "Cross-branch message approval"
      }
    ]
  },
  {
    "id": "states-and-refusals",
    "owner": "figcraft-hivemind",
    "title": "10 · States & Refusals",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-states-and-refusals.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-states-and-refusals.svg",
    "sha256": "580561355febbe4f73c1635464bced7b1a7007505369e4f53941b27eb12ae7f8",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "proposed",
      "verdict": null,
      "ledger": null,
      "basis": "PROPOSED-ADDITION / motion proposal per docs/ui/figcraft/HiveMind/README.md; not watchdog-audited"
    },
    "surfaces": [
      {
        "surfaceId": "S14",
        "name": "Refusal card / named refusal"
      },
      {
        "surfaceId": "S25",
        "name": "Code-diff review (DECISION-GATED)"
      },
      {
        "surfaceId": "S26",
        "name": "First-run / empty / loading states"
      },
      {
        "surfaceId": "STATE-RECONNECT",
        "name": "Reconnect state"
      },
      {
        "surfaceId": "STATE-KILL-CONFIRM",
        "name": "Kill confirmation"
      },
      {
        "surfaceId": "STATE-REPLAY",
        "name": "Replay read-only"
      },
      {
        "surfaceId": "STATE-DIFF-GATED",
        "name": "DIFF surface explanation"
      }
    ]
  },
  {
    "id": "motion-and-live",
    "owner": "figcraft-hivemind",
    "title": "11 · Motion & Live",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-motion-and-live.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-motion-and-live.svg",
    "sha256": "c5e8b0a582dc4c9af061d0a30a388cb7142a0a3121f54949500f74cd05868998",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "proposed",
      "verdict": null,
      "ledger": null,
      "basis": "PROPOSED-ADDITION / motion proposal per docs/ui/figcraft/HiveMind/README.md; not watchdog-audited"
    },
    "surfaces": [
      {
        "surfaceId": "CHAT-VARIANT-400x480",
        "name": "Chat core app variant 400x480"
      },
      {
        "surfaceId": "MOTION-AMBIENT",
        "name": "Honeycomb ambient glow sweep (18s linear, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-SHEEN",
        "name": "5-lobe hive sheen (6s ease-in-out, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-BEE-FLOW",
        "name": "Bee flow with arrowheads (2.4s cubic-bezier, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-PULSE",
        "name": "Running state pulse (1.6s, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-SPINNER",
        "name": "Working-bee spinner (1.2s linear, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-RISE-HOVER",
        "name": "Node rise/hover (240ms / 120ms, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-FOCUS-RING",
        "name": "Focus ring ink-700 2px (160ms, PROPOSED)"
      },
      {
        "surfaceId": "MOTION-REDUCED",
        "name": "Reduced-motion fallback (PROPOSED)"
      }
    ]
  },
  {
    "id": "run-detail-and-files",
    "owner": "figcraft-hivemind",
    "title": "12 · Run Detail & Files",
    "width": 1440,
    "height": 900,
    "svgFile": "hivemind-run-detail-and-files.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/hivemind-run-detail-and-files.svg",
    "sha256": "80afcd4f598a4e18dd1c8275fd687c2d86497d574f0ba863bfccaef03aba4167",
    "manifest": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "audit": {
      "status": "proposed",
      "verdict": null,
      "ledger": null,
      "basis": "PROPOSED-ADDITION / motion proposal per docs/ui/figcraft/HiveMind/README.md; not watchdog-audited"
    },
    "surfaces": [
      {
        "surfaceId": "DATA-FILES",
        "name": "Data & files read-only tree (SAMPLE/READ)"
      },
      {
        "surfaceId": "FILE-PREVIEW",
        "name": "FILE_READ honest empty preview"
      },
      {
        "surfaceId": "TOOL-ACTIVITY",
        "name": "Tool activity ledger rows (no fabricated calls)"
      },
      {
        "surfaceId": "RUN-DETAIL",
        "name": "Run detail: id/state/timestamps/revision/stages"
      },
      {
        "surfaceId": "FAILURE-LEDGER",
        "name": "Failure / refusal ledger with named codes"
      },
      {
        "surfaceId": "MODEL-STATUS",
        "name": "Model/provider status"
      },
      {
        "surfaceId": "USAGE-COUNTERS",
        "name": "Six usage counters (prompt/completion/total/cache hit/cache miss/reasoning)"
      },
      {
        "surfaceId": "SETTINGS-KEY",
        "name": "Key-present / missing status"
      },
      {
        "surfaceId": "SETTINGS-REDUCED-MOTION",
        "name": "Reduced-motion setting (PROPOSED)"
      },
      {
        "surfaceId": "SETTINGS-DIFF-GATED",
        "name": "No diff control (DECISION-GATED)"
      }
    ]
  },
  {
    "id": "a-specimens",
    "owner": "figcraft-a",
    "title": "03 Specimens",
    "width": 1440,
    "height": 1400,
    "svgFile": "a-specimens.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/a-specimens.svg",
    "sha256": "e88aeca9a0f448fe87116837e11f560d202f1d25066d441a7b591e1f70dbe350",
    "manifest": "docs/ui/figcraft/A/A-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": [
      {
        "surfaceId": "S01",
        "name": "Infinite canvas + honeycomb background"
      },
      {
        "surfaceId": "S25",
        "name": "Code-diff review modal (no artboard)"
      },
      {
        "surfaceId": "S26",
        "name": "First-run / empty / loading / offline states"
      },
      {
        "surfaceId": "S27",
        "name": "Streamed reply / cards"
      }
    ]
  },
  {
    "id": "a-components",
    "owner": "figcraft-a",
    "title": "02 Components",
    "width": 1440,
    "height": 2580,
    "svgFile": "a-components.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/a-components.svg",
    "sha256": "2eb182185e22183c3473f1370784e2834c7646e7648c4d2feba95bdf3adbe0e9",
    "manifest": "docs/ui/figcraft/A/A-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": [
      {
        "surfaceId": "S02",
        "name": "HyperAgent hive chat core (pinned)"
      },
      {
        "surfaceId": "S03",
        "name": "Composer / HyperAgent input"
      },
      {
        "surfaceId": "S04",
        "name": "Button: Spawn sub-hive"
      },
      {
        "surfaceId": "S05",
        "name": "Button: Re-route"
      },
      {
        "surfaceId": "S06",
        "name": "Button: Cancel"
      },
      {
        "surfaceId": "S07",
        "name": "Button: Kill worker"
      },
      {
        "surfaceId": "S08",
        "name": "Tabs: Workspace / Swarm / Routing / History"
      },
      {
        "surfaceId": "S09",
        "name": "HUD throughput badge"
      },
      {
        "surfaceId": "S10",
        "name": "Time-travel scrubber"
      },
      {
        "surfaceId": "S11",
        "name": "Nav rail (72px, 6 items)"
      },
      {
        "surfaceId": "S12",
        "name": "Library panel (260px)"
      },
      {
        "surfaceId": "S13",
        "name": "Mini-map (260x170)"
      },
      {
        "surfaceId": "S14",
        "name": "Refusal toast"
      },
      {
        "surfaceId": "S15",
        "name": "SuperAgent sub-hive card"
      },
      {
        "surfaceId": "S16",
        "name": "WorkerAgent card"
      },
      {
        "surfaceId": "S17",
        "name": "ToolAgent card"
      },
      {
        "surfaceId": "S18",
        "name": "Broken-route error card"
      },
      {
        "surfaceId": "S19",
        "name": "Agent chips / state dot"
      },
      {
        "surfaceId": "S20",
        "name": "Zoom controls + shortcut hint"
      },
      {
        "surfaceId": "S21",
        "name": "Settings / key status"
      },
      {
        "surfaceId": "S22",
        "name": "File/code hierarchy viewer"
      },
      {
        "surfaceId": "S28",
        "name": "Project identity"
      }
    ]
  },
  {
    "id": "a-brand",
    "owner": "figcraft-a",
    "title": "01 Brand",
    "width": 1440,
    "height": 900,
    "svgFile": "a-brand.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/a-brand.svg",
    "sha256": "94cf946703c14355601813367f9a5fa11105d4e744990dd66ac7b0e5ad0a9522",
    "manifest": "docs/ui/figcraft/A/A-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": [
      {
        "surfaceId": "S23",
        "name": "Brand lockups / bee glyph"
      }
    ]
  },
  {
    "id": "a-foundations",
    "owner": "figcraft-a",
    "title": "00 Foundations",
    "width": 1440,
    "height": 1200,
    "svgFile": "a-foundations.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/a-foundations.svg",
    "sha256": "24536354e6ffe9ba4dda2c9682b526bb44bedea6aea7f1d21fd4e3aeb75e551a",
    "manifest": "docs/ui/figcraft/A/A-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": [
      {
        "surfaceId": "S24",
        "name": "Elevation e1/e2/e3"
      }
    ]
  },
  {
    "id": "b-canvas-default",
    "owner": "figcraft-b",
    "title": "Canvas Default",
    "width": 1440,
    "height": 900,
    "svgFile": "b-canvas-default.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/b-canvas-default.svg",
    "sha256": "b5a5a824145a85848247a10a663fab2d4783cc120d04de51fb93316a9ee33d9f",
    "manifest": "docs/ui/figcraft/B/B-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": [
      {
        "surfaceId": "S01",
        "name": "Infinite canvas + honeycomb lattice"
      },
      {
        "surfaceId": "S10",
        "name": "Time-travel scrubber"
      },
      {
        "surfaceId": "S13",
        "name": "Mini-map 260x170"
      },
      {
        "surfaceId": "S19",
        "name": "Agent chips / state dot"
      },
      {
        "surfaceId": "S20",
        "name": "Zoom controls + Ctrl+0 hint"
      },
      {
        "surfaceId": "S02",
        "name": "HyperAgent hive chat core (pinned)"
      },
      {
        "surfaceId": "S03",
        "name": "Composer / HyperAgent input"
      },
      {
        "surfaceId": "S04",
        "name": "Button: Spawn sub-hive"
      },
      {
        "surfaceId": "S05",
        "name": "Button: Re-route"
      },
      {
        "surfaceId": "S06",
        "name": "Button: Cancel"
      },
      {
        "surfaceId": "S07",
        "name": "Button: Kill worker"
      },
      {
        "surfaceId": "S08",
        "name": "Tabs: Workspace/Swarm/Routing/History"
      },
      {
        "surfaceId": "S09",
        "name": "HUD throughput badge"
      },
      {
        "surfaceId": "S11",
        "name": "Nav rail 72px, 6 items"
      },
      {
        "surfaceId": "S12",
        "name": "Library panel 260px"
      },
      {
        "surfaceId": "S15",
        "name": "SuperAgent sub-hive card"
      },
      {
        "surfaceId": "S16",
        "name": "WorkerAgent card"
      },
      {
        "surfaceId": "S17",
        "name": "ToolAgent card"
      },
      {
        "surfaceId": "S18",
        "name": "Broken-route error card"
      },
      {
        "surfaceId": "S21",
        "name": "Settings / key status"
      },
      {
        "surfaceId": "S22",
        "name": "File/code hierarchy viewer"
      },
      {
        "surfaceId": "S23",
        "name": "Brand lockups / bee glyph"
      },
      {
        "surfaceId": "S27",
        "name": "Streamed reply / cards"
      },
      {
        "surfaceId": "S28",
        "name": "Project identity"
      }
    ]
  },
  {
    "id": "b-zoom-states",
    "owner": "figcraft-b",
    "title": "Zoom States",
    "width": 1440,
    "height": 900,
    "svgFile": "b-zoom-states.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/b-zoom-states.svg",
    "sha256": "ab1ee97ae2b9d042b9d8b78ef22f10f1d4b594041fbc51acd27c32112a9493ae",
    "manifest": "docs/ui/figcraft/B/B-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": []
  },
  {
    "id": "b-interaction",
    "owner": "figcraft-b",
    "title": "Interaction",
    "width": 1440,
    "height": 900,
    "svgFile": "b-interaction.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/b-interaction.svg",
    "sha256": "927514772ac5321c3771ed7293ae72b67e53526cb544222b21af408550a2e945",
    "manifest": "docs/ui/figcraft/B/B-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": [
      {
        "surfaceId": "S24",
        "name": "Elevation e1/e2/e3"
      },
      {
        "surfaceId": "S25",
        "name": "Code-diff review modal (decision-gated)"
      },
      {
        "surfaceId": "S14",
        "name": "Refusal toast"
      }
    ]
  },
  {
    "id": "b-hero",
    "owner": "figcraft-b",
    "title": "Hero",
    "width": 1440,
    "height": 900,
    "svgFile": "b-hero.svg",
    "sourcePath": "docs/ui/figcraft/ASSETS/b-hero.svg",
    "sha256": "2bf8d20facba986b2c148702a774187538886658c680f7d323aa0e785a678477",
    "manifest": "docs/ui/figcraft/B/B-ASSETS.json",
    "audit": {
      "status": "watchdog-audited",
      "verdict": "PASS",
      "ledger": "ledgers/ui-watchdog.json",
      "basis": "watchdog-audited export, sha verified against the ledger"
    },
    "surfaces": []
  }
];

export const FIGCRAFT_COUNTS = {
  "boards": 21,
  "assets": 114,
  "surfaces": 60,
  "surfaceMappings": 114,
  "distinctSurfaces": 60,
  "owners": 3,
  "watchdogAuditedBoards": 8,
  "selfVerifiedBoards": 8,
  "proposedBoards": 5,
  "unverifiedBoards": 0,
  "watchdog": "PASS",
  "allowFrontend": true
} as const;

export const FIGCRAFT_SOURCE_MANIFESTS: readonly FigCraftSourceManifest[] = [
  {
    "name": "ASSETS-INDEX",
    "path": "docs/ui/figcraft/ASSETS-INDEX.json",
    "sha256": "1C2426CCC208B149465D5FDFCCACDB7E57A3F2AFC2E729EAF4767C551648BFC0"
  },
  {
    "name": "A",
    "path": "docs/ui/figcraft/A/A-ASSETS.json",
    "sha256": "88832272CB6C81DDFF234D77F5ABBDF4D9811AE1E8D787B7B02F1C07F11E77F1"
  },
  {
    "name": "B",
    "path": "docs/ui/figcraft/B/B-ASSETS.json",
    "sha256": "A8A81E74BFBFBB3B9BCA9150DB734E8A1444D9B9E04F82CEF5C71A17F55D1F4D"
  },
  {
    "name": "HiveMind",
    "path": "docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json",
    "sha256": "4F0C928C41438F0CEB7D9C7D0F7FE2F68918AE35674B14FAC1FD827C7BF639FC"
  }
];
