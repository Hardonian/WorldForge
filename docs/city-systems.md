# City systems, research, and governance

World Forge worlds may include an optional `city.toml`. This turns the deterministic resource network into a player-directed city: construction and research consume real inventory, buildings participate in the tick economy, and every decision is included in saves, replays, event-chain proofs, and state fingerprints.

## Gameplay loop

1. Build inside finite district slot budgets.
2. Pay construction costs from the configured treasury entity.
3. Sustain each building's per-tick upkeep or lose its output for that tick.
4. Combine tagged buildings in one district to activate configured output synergies.
5. Generate research and choose paths through the technology graph.
6. Use technology effects to alter building- and resource-specific yields, housing, jobs, and wellbeing.
7. Grow population toward constructed housing capacity while paying per-resident needs.
8. Resolve timed civic dilemmas whose choices create permanent economic, spatial, and political consequences.
9. Navigate rival realms, tribute, coalitions, border threats, and humanitarian crises.
10. Run corporate intelligence operations, contain rogue agents, acquire trade secrets, and trade or manipulate volatile token markets.

Research supports prerequisites, cross-branch synthesis, and mutually exclusive choices. The engine validates references and rejects prerequisite cycles before a world can run.

## Civic governance

`[[factions]]` define named constituencies with support from 0–100. `[[dilemmas]]` open when deterministic trigger conditions are met: a tick threshold, treasury resources above or below thresholds, completed technologies, prior choices, or excluded choices. Dilemmas contain at least two options and may specify a deadline plus a cost-free default.

An option can spend or grant treasury resources, shift faction support, and permanently multiply housing, jobs, population growth, building yields, or resource yields. It can also add wellbeing. Later dilemmas can require a specific prior choice, allowing authored campaigns to branch without scripting. Choice dependency cycles and dangling faction, technology, building, dilemma, and option references fail validation.

## Geopolitics and shadow economy

`[[political_entities]]` model fiefdoms, vassals, hegemons, coalitions, and insurgencies with stance, loyalty, military power, traits, and tribute. Realm actions can demand tribute, send emissaries, establish coalitions, fortify the city, or strike hostile forces. Border pressure, famine risk, refugee flows, and civic dilemmas connect external power to the city economy.

`[[corporations]]` own guarded `[[corporations.secrets]]`; `[[cyber_agents]]` have skill, stealth, loyalty, containment, and a live operational status; `[[crypto_assets]]` define price, volatility, and liquidity. Abstract infiltration contests agent capability against corporate security and secret difficulty. Success grants research, while attribution raises heat and can compromise an agent or send it rogue. Rogue agents periodically siphon city resources until contained. Token prices move on independent deterministic streams; players can buy, sell, pump, or dump, with manipulation creating heat.

These are fictional strategy abstractions designed to teach incentives, risk, feedback, attribution, and second-order consequences. They do not model real intrusion procedures.

## Runtime contract

`city.toml` identifies one existing entity as its treasury. Player actions are atomic: the engine verifies unlocks, exclusions, affordability, district compatibility, free slots, and count limits before changing state. Fixed-point accounting is used after configuration values enter the simulation.

The live API exposes the effective city state in `city` and accepts:

- `POST /api/play/sessions/{id}/construct` with `{ "building": "...", "district": "..." }`
- `POST /api/play/sessions/{id}/research` with `{ "technology": "..." }`
- `POST /api/play/sessions/{id}/decide` with `{ "dilemma": "...", "option": "..." }`
- `POST /api/play/sessions/{id}/geopolitics` with `{ "action": "...", "entity": "...", "option": null }`
- `POST /api/play/sessions/{id}/intrigue` with `{ "action": "...", "target": "...", "agent": null, "option": null }`

The existing intervention endpoint remains available for production-capacity decisions. Every action is restored in deterministic tick order when loading a save. Dilemma openings, political orders, covert outcomes, market moves, rogue incidents, and player decisions are proof-chained events.

## Content inheritance

A derived world inherits its last resolved base world's `city.toml`. A local `city.toml` replaces that city layer as a complete ruleset, while the normal entity graph still uses named overlays. This keeps research graphs internally coherent and makes an intentional ruleset fork obvious in package review.

See `examples/micro-city/city.toml` for a full four-district world with a large technology tree, civic factions and crises, geopolitical actors, three rival corporations, six trade secrets, three cyber agents, and three crypto markets. `schemas/city.schema.json` documents the serialized shape; engine validation remains authoritative for graph and cross-file constraints.
