/** World Forge Simulation Studio — dependency-free dashboard client. */

const COLORS = ['#67a9ff', '#42d3ea', '#9b8cff', '#46d29a', '#f1b96b', '#ff6f7c', '#d578ef'];
const TYPE_COLORS = {
    production: '#46d29a',
    transfer: '#67a9ff',
    shortage: '#ff6f7c',
    price: '#f1b96b',
    system: '#9b8cff',
};
const WORLD_SYMBOLS = {
    'supply-chain': 'SC', ecosystem: 'EC', 'micro-city': 'MC',
    'freight-network': 'FN', 'stress-test': 'ST', 'minimal-world': 'MW',
};
const HISTORY_KEY = 'worldforge.recent-runs.v1';
const MAX_HISTORY = 7;
const EVENTS_PAGE_SIZE = 100;

const state = {
    worlds: [],
    worldId: null,
    data: null,
    eventsVisible: EVENTS_PAGE_SIZE,
    history: loadHistory(),
    busy: false,
};

const el = id => document.getElementById(id);
const dom = {
    body: document.body,
    worldNav: el('world-nav'), worldCount: el('world-count'),
    worldTitle: el('world-title'), worldDesc: el('world-desc'), worldMeta: el('world-meta'),
    seed: el('seed-input'), ticks: el('ticks-input'), runForm: el('run-form'), runButton: el('run-btn'), runLabel: el('run-label'),
    exportButton: el('export-btn'), resourceSelect: el('resource-select'),
    loading: el('loading-overlay'), loadingTitle: el('loading-title'), loadingCopy: el('loading-copy'),
    eventLog: el('event-log'), eventFilter: el('log-filter'), eventType: el('log-type-filter'),
    eventFooter: el('event-footer'), eventCountLabel: el('event-count-label'), loadMore: el('load-more-events'),
    compareDialog: el('compare-dialog'), benchmarkDialog: el('benchmark-dialog'),
};

function currentWorld() {
    return state.worlds.find(world => world.id === state.worldId) || null;
}

async function api(path, options = {}) {
    const response = await fetch(path, options);
    let payload;
    try {
        payload = await response.json();
    } catch {
        payload = {};
    }
    if (!response.ok) {
        const error = new Error(payload.error || `Request failed with status ${response.status}`);
        error.code = payload.errorId || payload.code;
        throw error;
    }
    return payload;
}

async function initialize() {
    bindInteractions();
    renderHistory();
    try {
        const [health, catalog] = await Promise.all([api('/api/health'), api('/api/worlds')]);
        setEngineStatus(true, `Engine v${health.engineVersion}`, `${health.worlds} worlds available`);
        el('footer-version').textContent = `v${health.engineVersion}`;
        state.worlds = catalog.worlds || [];
        renderWorldNavigation();
        if (!state.worlds.length) throw new Error('No packaged worlds were found.');
        selectWorld(state.worlds[0].id, { useDefaults: true });
    } catch (error) {
        setEngineStatus(false, 'Engine offline', 'Dashboard API unavailable');
        renderCatalogError(error.message);
        showToast('Could not connect', `${error.message} Start the dashboard through the World Forge CLI.`, 'error');
    }
}

function bindInteractions() {
    dom.runForm.addEventListener('submit', event => {
        event.preventDefault();
        runSimulation();
    });
    dom.exportButton.addEventListener('click', exportCurrentRun);
    dom.resourceSelect.addEventListener('change', () => state.data && drawResourceChart(state.data));
    dom.eventFilter.addEventListener('input', resetAndRenderEvents);
    dom.eventType.addEventListener('change', resetAndRenderEvents);
    dom.loadMore.addEventListener('click', () => {
        state.eventsVisible += EVENTS_PAGE_SIZE;
        renderEvents();
    });
    el('nav-compare').addEventListener('click', openComparison);
    el('nav-benchmark').addEventListener('click', openBenchmark);
    el('compare-run').addEventListener('click', runComparison);
    el('benchmark-run').addEventListener('click', runBenchmark);
    el('clear-history').addEventListener('click', clearHistory);
    el('menu-button').addEventListener('click', () => toggleSidebar(true));
    el('sidebar-close').addEventListener('click', () => toggleSidebar(false));
    el('mobile-backdrop').addEventListener('click', () => toggleSidebar(false));
    el('val-fingerprint').addEventListener('click', () => copyProof('final'));
    document.querySelectorAll('[data-proof]').forEach(button => {
        button.addEventListener('click', () => copyProof(button.dataset.proof));
    });
    document.querySelectorAll('.tool-dialog').forEach(dialog => {
        dialog.addEventListener('click', event => {
            if (event.target === dialog) dialog.close();
        });
    });
    document.addEventListener('keydown', event => {
        if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
            event.preventDefault();
            runSimulation();
        }
    });
    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(redrawCharts, 120);
    });
}

function setEngineStatus(online, label, detail) {
    el('engine-dot').className = `status-dot ${online ? 'online' : 'offline'}`;
    el('engine-label').textContent = label;
    el('engine-version').textContent = detail;
}

function renderCatalogError(message) {
    dom.worldNav.replaceChildren();
    const empty = document.createElement('p');
    empty.className = 'sidebar-empty';
    empty.textContent = message;
    dom.worldNav.append(empty);
    dom.worldCount.textContent = '0';
}

function renderWorldNavigation() {
    dom.worldNav.replaceChildren();
    dom.worldCount.textContent = state.worlds.length.toString();
    state.worlds.forEach((world, index) => {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'nav-item';
        button.dataset.world = world.id;
        const icon = document.createElement('span');
        icon.className = 'nav-icon world-symbol';
        icon.textContent = WORLD_SYMBOLS[world.id] || String(index + 1).padStart(2, '0');
        const copy = document.createElement('span');
        copy.className = 'nav-copy';
        const title = document.createElement('strong');
        title.textContent = world.title;
        const details = document.createElement('small');
        details.textContent = `${world.entityCount} entities · ${world.resourceCount} resources`;
        copy.append(title, details);
        const arrow = document.createElement('span');
        arrow.className = 'nav-arrow';
        arrow.setAttribute('aria-hidden', 'true');
        arrow.textContent = '→';
        button.append(icon, copy, arrow);
        button.addEventListener('click', () => {
            selectWorld(world.id, { useDefaults: true });
            toggleSidebar(false);
        });
        dom.worldNav.append(button);
    });
}

function selectWorld(worldId, options = {}) {
    const world = state.worlds.find(item => item.id === worldId);
    if (!world) return;
    state.worldId = worldId;
    state.data = null;
    document.querySelectorAll('[data-world]').forEach(button => {
        const active = button.dataset.world === worldId;
        button.classList.toggle('active', active);
        button.setAttribute('aria-current', active ? 'page' : 'false');
    });
    dom.worldTitle.textContent = world.title;
    dom.worldDesc.textContent = world.description || 'A packaged deterministic simulation world.';
    dom.worldMeta.textContent = `v${world.version} · ${world.entityCount} entities · ${world.resourceCount} resources`;
    if (options.useDefaults) {
        dom.seed.value = world.defaultSeed;
        dom.ticks.value = world.defaultTicks;
    }
    fillResourceSelect(world.resources);
    el('network-meta').textContent = `${world.links.length} link${world.links.length === 1 ? '' : 's'}`;
    resetResults();
    drawEntityNetwork(world);
}

function fillResourceSelect(resources) {
    dom.resourceSelect.replaceChildren(new Option('All resources', 'all'));
    resources.forEach(resource => dom.resourceSelect.add(new Option(prettyName(resource), resource)));
}

function numericInput(input, { min, max, name }) {
    const value = Number(input.value);
    if (!Number.isFinite(value) || !Number.isInteger(value) || value < min || value > max) {
        input.focus();
        throw new Error(`${name} must be a whole number between ${min.toLocaleString()} and ${max.toLocaleString()}.`);
    }
    return value;
}

async function runSimulation() {
    if (state.busy || !currentWorld()) return;
    let seed, ticks;
    try {
        seed = numericInput(dom.seed, { min: 0, max: Number.MAX_SAFE_INTEGER, name: 'Seed' });
        ticks = numericInput(dom.ticks, { min: 1, max: 1_000_000, name: 'Ticks' });
    } catch (error) {
        showToast('Check the run settings', error.message, 'error');
        return;
    }

    setMainBusy(true, 'Forging simulation', `Advancing ${ticks.toLocaleString()} deterministic ticks…`);
    const started = performance.now();
    try {
        const data = await api('/api/run', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ world: state.worldId, seed, ticks }),
        });
        state.data = data;
        displayResults(data);
        addHistory(data, performance.now() - started);
        showToast('Simulation complete', `${formatNumber(data.totalEvents)} events forged across ${formatNumber(data.ticks)} ticks.`);
    } catch (error) {
        showToast('Simulation failed', `${error.code ? `${error.code}: ` : ''}${error.message}`, 'error');
    } finally {
        setMainBusy(false);
    }
}

function setMainBusy(busy, title = '', copy = '') {
    state.busy = busy;
    dom.runButton.disabled = busy;
    dom.runButton.classList.toggle('running', busy);
    dom.runLabel.textContent = busy ? 'Running…' : 'Run simulation';
    dom.loadingTitle.textContent = title;
    dom.loadingCopy.textContent = copy;
    dom.loading.classList.toggle('hidden', !busy);
}

function displayResults(data) {
    const world = currentWorld();
    if (world) {
        world.entities = data.entities || world.entities;
        world.resources = data.resources || world.resources;
        world.links = data.links || world.links;
        fillResourceSelect(world.resources);
        drawEntityNetwork(world);
    }
    el('val-events').textContent = formatNumber(data.totalEvents);
    el('val-shortages').textContent = formatNumber(data.shortageEvents);
    el('trend-events').textContent = `${formatNumber(data.ticks)} ticks · seed ${data.seed}`;
    const shortageRate = data.totalEvents ? data.shortageEvents / data.totalEvents * 100 : 0;
    el('trend-shortages').textContent = `${shortageRate.toFixed(shortageRate < 1 ? 2 : 1)}% of all events`;
    const fingerprintButton = el('val-fingerprint');
    fingerprintButton.textContent = shortHash(data.fingerprints.final);
    fingerprintButton.disabled = false;
    fingerprintButton.title = `Copy ${data.fingerprints.final}`;
    el('val-chain').textContent = 'Verified';
    el('val-chain').classList.add('verified');
    el('proof-summary').textContent = `${data.meta.runId.slice(0, 8)} · chain intact`;
    dom.exportButton.disabled = false;

    const passed = data.objectives.filter(objective => objective.status.toLowerCase() === 'passed').length;
    el('run-banner').classList.add('complete');
    el('run-banner-title').textContent = `${data.title} completed with a verified proof.`;
    el('run-banner-copy').textContent = `Run ${data.meta.runId} completed.`;
    el('objective-summary').textContent = `${passed} of ${data.objectives.length} passed`;
    renderObjectives(data.objectives);
    renderProof(data.fingerprints);
    configureEventTypes(data.eventTypeCounts);
    state.eventsVisible = EVENTS_PAGE_SIZE;
    renderEvents();
    drawResourceChart(data);
    drawEventDistribution(data);
}

function resetResults() {
    state.eventsVisible = EVENTS_PAGE_SIZE;
    ['val-events', 'val-shortages'].forEach(id => el(id).textContent = '—');
    el('trend-events').textContent = 'Awaiting run';
    el('trend-shortages').textContent = 'Awaiting run';
    el('val-fingerprint').textContent = '—';
    el('val-fingerprint').disabled = true;
    el('val-chain').textContent = 'Not run';
    el('val-chain').classList.remove('verified');
    el('proof-summary').textContent = 'Proof generated on completion';
    el('run-banner').classList.remove('complete');
    el('run-banner-title').textContent = 'A reproducible world, down to the event.';
    el('run-banner-copy').textContent = 'Every result is calculated by the Rust runtime. Run the selected world to reveal resource flows, objectives, event history, and a tamper-evident proof chain.';
    el('objectives-list').replaceChildren(emptyState('◇', '', 'Objectives will be evaluated across the whole run.', true));
    el('objective-summary').textContent = 'Not evaluated';
    el('distribution-meta').textContent = 'No run';
    dom.eventFilter.value = '';
    dom.eventType.replaceChildren(new Option('All event types', 'all'));
    dom.eventLog.replaceChildren(emptyState('⌁', 'No events yet', 'Run a simulation to inspect its chronological audit trail.'));
    dom.eventFooter.classList.add('hidden');
    dom.exportButton.disabled = true;
    document.querySelectorAll('[data-proof]').forEach(button => button.disabled = true);
    ['proof-world', 'proof-initial', 'proof-final', 'proof-chain'].forEach(id => el(id).textContent = '—');
    el('verified-chip').classList.remove('verified');
    el('verified-chip').lastChild.textContent = 'Awaiting run';
    el('resource-empty').classList.remove('hidden');
    el('event-empty').classList.remove('hidden');
    clearCanvas('resource-chart');
    clearCanvas('event-chart');
}

function emptyState(mark, title, copy, compact = false) {
    const node = document.createElement('div');
    node.className = `empty-state${compact ? ' compact' : ''}`;
    const glyph = document.createElement('span');
    glyph.className = 'empty-mark';
    glyph.textContent = mark;
    node.append(glyph);
    if (title) {
        const heading = document.createElement('h3');
        heading.textContent = title;
        node.append(heading);
    }
    const paragraph = document.createElement('p');
    paragraph.textContent = copy;
    node.append(paragraph);
    return node;
}

function renderObjectives(objectives) {
    const container = el('objectives-list');
    container.replaceChildren();
    if (!objectives.length) {
        container.append(emptyState('◇', '', 'This scenario has no configured objectives.', true));
        return;
    }
    objectives.forEach(objective => {
        const status = objective.status.toLowerCase();
        const card = document.createElement('article');
        card.className = `objective-card ${status}`;
        const icon = document.createElement('span');
        icon.className = 'objective-icon';
        icon.textContent = status === 'passed' ? '✓' : status === 'failed' ? '×' : '…';
        const copy = document.createElement('span');
        copy.className = 'objective-copy';
        const name = document.createElement('strong');
        name.textContent = objective.name;
        const detail = document.createElement('small');
        detail.textContent = objective.status;
        copy.append(name, detail);
        card.append(icon, copy);
        container.append(card);
    });
}

function renderProof(fingerprints) {
    const mapping = { world: 'proof-world', initial: 'proof-initial', final: 'proof-final', eventChain: 'proof-chain' };
    Object.entries(mapping).forEach(([key, id]) => {
        el(id).textContent = shortHash(fingerprints[key], 18);
        const button = document.querySelector(`[data-proof="${key}"]`);
        button.disabled = false;
        button.title = fingerprints[key];
    });
    const chip = el('verified-chip');
    chip.classList.add('verified');
    chip.lastChild.textContent = 'Chain verified';
}

function configureEventTypes(counts) {
    const selected = dom.eventType.value;
    dom.eventType.replaceChildren(new Option('All event types', 'all'));
    Object.keys(counts).filter(type => counts[type] > 0).forEach(type => {
        dom.eventType.add(new Option(`${prettyName(type)} (${formatNumber(counts[type])})`, type));
    });
    if ([...dom.eventType.options].some(option => option.value === selected)) dom.eventType.value = selected;
}

function resetAndRenderEvents() {
    state.eventsVisible = EVENTS_PAGE_SIZE;
    renderEvents();
}

function filteredEvents() {
    if (!state.data) return [];
    const query = dom.eventFilter.value.trim().toLowerCase();
    const type = dom.eventType.value;
    return state.data.events.filter(event => {
        const matchesType = type === 'all' || event.type === type;
        const haystack = `${event.tick} ${event.type} ${event.entity} ${event.resource} ${event.summary}`.toLowerCase();
        return matchesType && (!query || haystack.includes(query));
    });
}

function renderEvents() {
    const events = filteredEvents();
    dom.eventLog.replaceChildren();
    if (!events.length) {
        dom.eventLog.append(emptyState('⌕', 'No matching events', state.data ? 'Change the search or event type filter.' : 'Run a simulation to inspect events.'));
        dom.eventFooter.classList.add('hidden');
        return;
    }
    events.slice(0, state.eventsVisible).forEach(event => {
        const row = document.createElement('div');
        row.className = 'log-entry';
        const tick = document.createElement('span');
        tick.className = 'log-tick';
        tick.textContent = `t${formatNumber(event.tick)}`;
        const type = document.createElement('span');
        type.className = `log-type ${event.type}`;
        type.textContent = event.type;
        const detail = document.createElement('span');
        detail.className = 'log-detail';
        detail.textContent = event.summary || `${event.entity} · ${event.resource}`;
        detail.title = detail.textContent;
        const amount = document.createElement('span');
        amount.className = 'log-amount';
        amount.textContent = event.amount ? formatDecimal(event.amount) : '—';
        row.append(tick, type, detail, amount);
        dom.eventLog.append(row);
    });
    dom.eventFooter.classList.remove('hidden');
    dom.eventCountLabel.textContent = `Showing ${Math.min(events.length, state.eventsVisible).toLocaleString()} of ${events.length.toLocaleString()} matching events`;
    dom.loadMore.classList.toggle('hidden', state.eventsVisible >= events.length);
}

function exportCurrentRun() {
    if (!state.data) return;
    const body = JSON.stringify(state.data, null, 2);
    const url = URL.createObjectURL(new Blob([body], { type: 'application/json' }));
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${state.data.world}-seed-${state.data.seed}-${state.data.ticks}-ticks.json`;
    document.body.append(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(url);
    showToast('Run exported', 'The canonical simulation document was saved as JSON.');
}

async function copyProof(key) {
    if (!state.data?.fingerprints?.[key]) return;
    try {
        await navigator.clipboard.writeText(state.data.fingerprints[key]);
        showToast('Fingerprint copied', `${prettyName(key)} is ready to paste.`);
    } catch {
        showToast('Copy unavailable', 'Your browser did not grant clipboard access.', 'error');
    }
}

function addHistory(data, durationMs) {
    state.history.unshift({
        world: data.world, title: data.title, seed: data.seed, ticks: data.ticks,
        events: data.totalEvents, hash: data.fingerprints.final, durationMs,
        timestamp: Date.now(),
    });
    state.history = state.history.slice(0, MAX_HISTORY);
    saveHistory();
    renderHistory();
}

function loadHistory() {
    try {
        const history = JSON.parse(localStorage.getItem(HISTORY_KEY) || '[]');
        return Array.isArray(history) ? history.slice(0, MAX_HISTORY) : [];
    } catch { return []; }
}

function saveHistory() {
    try { localStorage.setItem(HISTORY_KEY, JSON.stringify(state.history)); } catch { /* private mode */ }
}

function renderHistory() {
    const container = el('recent-runs');
    container.replaceChildren();
    if (!state.history.length) {
        const empty = document.createElement('p');
        empty.className = 'sidebar-empty';
        empty.textContent = 'Completed runs appear here.';
        container.append(empty);
        return;
    }
    state.history.forEach(run => {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'recent-run';
        const dot = document.createElement('span'); dot.className = 'recent-run-dot';
        const copy = document.createElement('span'); copy.className = 'recent-run-copy';
        const title = document.createElement('strong'); title.textContent = run.title;
        const meta = document.createElement('small'); meta.textContent = `seed ${run.seed} · ${formatNumber(run.events)} events`;
        copy.append(title, meta);
        const time = document.createElement('span'); time.className = 'recent-run-time'; time.textContent = relativeTime(run.timestamp);
        button.append(dot, copy, time);
        button.addEventListener('click', () => {
            if (state.worlds.some(world => world.id === run.world)) {
                selectWorld(run.world);
                dom.seed.value = run.seed;
                dom.ticks.value = run.ticks;
                toggleSidebar(false);
                showToast('Run settings restored', 'Press Run simulation to reproduce this result.');
            }
        });
        container.append(button);
    });
}

function clearHistory() {
    state.history = [];
    saveHistory();
    renderHistory();
    showToast('History cleared', 'Saved run summaries were removed from this browser.');
}

function openComparison() {
    if (!currentWorld()) return;
    el('compare-seed-a').value = dom.seed.value;
    el('compare-seed-b').value = Number(dom.seed.value || 42) + 1;
    el('compare-ticks').value = dom.ticks.value;
    dom.compareDialog.showModal();
}

async function runComparison() {
    let seedA, seedB, ticks;
    try {
        seedA = numericInput(el('compare-seed-a'), { min: 0, max: Number.MAX_SAFE_INTEGER, name: 'Baseline seed' });
        seedB = numericInput(el('compare-seed-b'), { min: 0, max: Number.MAX_SAFE_INTEGER, name: 'Candidate seed' });
        ticks = numericInput(el('compare-ticks'), { min: 1, max: 1_000_000, name: 'Ticks' });
    } catch (error) {
        showToast('Check comparison settings', error.message, 'error'); return;
    }
    const button = el('compare-run');
    setButtonBusy(button, true, 'Comparing…');
    const results = el('compare-results');
    results.replaceChildren(emptyState('◌', 'Running both simulations', 'The server is calculating two complete proof chains.'));
    try {
        const request = seed => api('/api/run', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ world: state.worldId, seed, ticks }) });
        const [a, b] = await Promise.all([request(seedA), request(seedB)]);
        renderComparison(a, b);
    } catch (error) {
        results.replaceChildren(emptyState('!', 'Comparison failed', error.message));
    } finally { setButtonBusy(button, false, 'Compare runs'); }
}

function renderComparison(a, b) {
    const results = el('compare-results');
    results.replaceChildren();
    const heading = document.createElement('div'); heading.className = 'result-heading';
    const title = document.createElement('strong'); title.textContent = `${a.title} · ${formatNumber(a.ticks)} ticks`;
    const detail = document.createElement('span'); detail.textContent = 'Engine-calculated results';
    heading.append(title, detail);
    const grid = document.createElement('div'); grid.className = 'comparison-grid';
    grid.append(comparisonColumn('Baseline', a), comparisonColumn('Candidate', b));
    const same = a.fingerprints.final === b.fingerprints.final && a.fingerprints.eventChain === b.fingerprints.eventChain;
    const verdict = document.createElement('div'); verdict.className = 'comparison-verdict';
    const mark = document.createElement('span'); mark.className = `verdict-mark${same ? '' : ' different'}`; mark.textContent = same ? '✓' : '≠';
    const copy = document.createElement('span');
    copy.textContent = same
        ? 'The final state and event chain match exactly. Reproducibility verified.'
        : `The seeds produced different valid outcomes: ${signedDifference(b.totalEvents - a.totalEvents)} events and ${signedDifference(b.shortageEvents - a.shortageEvents)} shortages.`;
    verdict.append(mark, copy);
    results.append(heading, grid, verdict);
}

function comparisonColumn(label, data) {
    const column = document.createElement('section'); column.className = 'comparison-run';
    const heading = document.createElement('h3'); heading.textContent = `${label} · seed ${data.seed}`;
    const list = document.createElement('div'); list.className = 'metric-list';
    [
        ['Events', formatNumber(data.totalEvents)],
        ['Shortages', formatNumber(data.shortageEvents)],
        ['Objectives', `${data.objectives.filter(item => item.status === 'Passed').length}/${data.objectives.length} passed`],
        ['Final state', shortHash(data.fingerprints.final, 16)],
        ['Event chain', shortHash(data.fingerprints.eventChain, 16)],
    ].forEach(([name, value]) => {
        const row = document.createElement('div'); row.className = 'metric-row';
        const key = document.createElement('span'); key.textContent = name;
        const val = document.createElement('strong'); val.textContent = value; val.title = value;
        row.append(key, val); list.append(row);
    });
    column.append(heading, list); return column;
}

function openBenchmark() {
    if (!currentWorld()) return;
    el('benchmark-ticks').value = dom.ticks.value;
    dom.benchmarkDialog.showModal();
}

async function runBenchmark() {
    let ticks, reps;
    try {
        ticks = numericInput(el('benchmark-ticks'), { min: 1, max: 1_000_000, name: 'Ticks' });
        reps = numericInput(el('benchmark-reps'), { min: 1, max: 20, name: 'Repetitions' });
    } catch (error) {
        showToast('Check benchmark settings', error.message, 'error'); return;
    }
    const button = el('benchmark-run');
    setButtonBusy(button, true, 'Benchmarking…');
    const results = el('benchmark-results');
    results.replaceChildren(emptyState('◌', 'Benchmark in progress', `Running ${reps} complete simulations on the server.`));
    try {
        const report = await api('/api/benchmark', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ world: state.worldId, ticks, reps }),
        });
        renderBenchmark(report);
    } catch (error) {
        results.replaceChildren(emptyState('!', 'Benchmark failed', error.message));
    } finally { setButtonBusy(button, false, 'Start benchmark'); }
}

function renderBenchmark(report) {
    const results = el('benchmark-results'); results.replaceChildren();
    const heading = document.createElement('div'); heading.className = 'result-heading';
    const title = document.createElement('strong'); title.textContent = `${currentWorld()?.title || report.world} performance`;
    const detail = document.createElement('span'); detail.textContent = `${report.reps} runs · ${formatNumber(report.ticks)} ticks each`;
    heading.append(title, detail);
    const kpis = document.createElement('div'); kpis.className = 'benchmark-kpis';
    [
        ['Average', `${formatDecimal(report.averageMs, 2)} ms`],
        ['Range', `${formatDecimal(report.minMs, 1)}–${formatDecimal(report.maxMs, 1)} ms`],
        ['Ticks / sec', compactNumber(report.ticksPerSecond)],
        ['Events / sec', compactNumber(report.eventsPerSecond)],
    ].forEach(([name, value]) => {
        const card = document.createElement('div'); card.className = 'benchmark-kpi';
        const label = document.createElement('span'); label.textContent = name;
        const metric = document.createElement('strong'); metric.textContent = value;
        card.append(label, metric); kpis.append(card);
    });
    const chart = document.createElement('div'); chart.className = 'sample-bars';
    const max = Math.max(...report.samplesMs, 1);
    report.samplesMs.forEach((sample, index) => {
        const wrap = document.createElement('div'); wrap.className = 'sample-bar-wrap';
        const bar = document.createElement('div'); bar.className = 'sample-bar';
        bar.style.height = `${Math.max(4, sample / max * 100)}%`;
        bar.dataset.value = `${formatDecimal(sample, 1)}ms`;
        const label = document.createElement('small'); label.textContent = `Run ${index + 1}`;
        wrap.append(bar, label); chart.append(wrap);
    });
    results.append(heading, kpis, chart);
}

function setButtonBusy(button, busy, label) {
    button.disabled = busy;
    button.textContent = label;
}

/* Canvas charts */
function prepareCanvas(id, height) {
    const canvas = el(id);
    const parent = canvas.parentElement;
    const width = Math.max(260, parent.clientWidth - 36);
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    const ctx = canvas.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);
    return { canvas, ctx, width, height };
}

function clearCanvas(id) {
    const canvas = el(id);
    canvas.getContext('2d').clearRect(0, 0, canvas.width, canvas.height);
}

function drawResourceChart(data) {
    const { ctx, width: w, height: h } = prepareCanvas('resource-chart', 300);
    el('resource-empty').classList.toggle('hidden', Boolean(data.snapshots?.length));
    if (!data.snapshots?.length) return;
    const selected = dom.resourceSelect.value;
    const allResources = data.resources || currentWorld()?.resources || [];
    const resources = selected === 'all' ? allResources.slice(0, 6) : [selected];
    const pad = { top: 28, right: 18, bottom: 50, left: 52 };
    const cw = w - pad.left - pad.right, ch = h - pad.top - pad.bottom;
    let maxValue = 0;
    data.snapshots.forEach(snapshot => resources.forEach(resource => {
        maxValue = Math.max(maxValue, Number(snapshot.levels[resource]) || 0);
    }));
    maxValue = niceMaximum(maxValue);

    ctx.lineWidth = 1;
    ctx.font = '10px ui-monospace, monospace';
    for (let i = 0; i <= 4; i++) {
        const y = pad.top + ch * i / 4;
        ctx.strokeStyle = 'rgba(163,184,214,.09)';
        ctx.beginPath(); ctx.moveTo(pad.left, y); ctx.lineTo(pad.left + cw, y); ctx.stroke();
        ctx.fillStyle = '#64738a'; ctx.textAlign = 'right';
        ctx.fillText(compactNumber(maxValue * (1 - i / 4)), pad.left - 8, y + 3);
    }
    const labels = Math.min(6, data.snapshots.length);
    for (let i = 0; i < labels; i++) {
        const index = Math.round(i * (data.snapshots.length - 1) / Math.max(1, labels - 1));
        const x = pad.left + cw * index / Math.max(1, data.snapshots.length - 1);
        ctx.fillStyle = '#64738a'; ctx.textAlign = 'center';
        ctx.fillText(`t${compactNumber(data.snapshots[index].tick)}`, x, h - 17);
    }
    resources.forEach((resource, resourceIndex) => {
        const color = COLORS[resourceIndex % COLORS.length];
        const points = data.snapshots.map((snapshot, index) => ({
            x: pad.left + cw * index / Math.max(1, data.snapshots.length - 1),
            y: pad.top + ch - (Number(snapshot.levels[resource]) || 0) / maxValue * ch,
        }));
        const gradient = ctx.createLinearGradient(0, pad.top, 0, pad.top + ch);
        gradient.addColorStop(0, `${color}27`); gradient.addColorStop(1, `${color}00`);
        ctx.beginPath(); points.forEach((point, index) => index ? ctx.lineTo(point.x, point.y) : ctx.moveTo(point.x, point.y));
        ctx.lineTo(points.at(-1).x, pad.top + ch); ctx.lineTo(points[0].x, pad.top + ch); ctx.closePath();
        ctx.fillStyle = gradient; ctx.fill();
        ctx.beginPath(); points.forEach((point, index) => index ? ctx.lineTo(point.x, point.y) : ctx.moveTo(point.x, point.y));
        ctx.strokeStyle = color; ctx.lineWidth = 1.7; ctx.lineJoin = 'round'; ctx.stroke();
    });
    let legendX = pad.left;
    ctx.font = '10px ui-sans-serif, sans-serif';
    resources.forEach((resource, index) => {
        const labelWidth = ctx.measureText(prettyName(resource)).width + 28;
        if (legendX + labelWidth > w - pad.right) return;
        ctx.fillStyle = COLORS[index % COLORS.length]; ctx.fillRect(legendX, 5, 12, 2);
        ctx.fillStyle = '#93a1b5'; ctx.textAlign = 'left'; ctx.fillText(prettyName(resource), legendX + 17, 9);
        legendX += labelWidth;
    });
}

function drawEntityNetwork(world) {
    if (!world) return;
    const { ctx, width: w, height: h } = prepareCanvas('network-chart', 260);
    const entities = world.entities || [], links = world.links || [];
    if (!entities.length) return;
    const cx = w / 2, cy = h / 2;
    const radius = Math.max(58, Math.min(w * .34, h * .34));
    const positions = entities.map((_, index) => ({
        x: cx + radius * Math.cos(index / entities.length * Math.PI * 2 - Math.PI / 2),
        y: cy + radius * Math.sin(index / entities.length * Math.PI * 2 - Math.PI / 2),
    }));
    links.forEach(link => {
        const fromIndex = entities.indexOf(link.from), toIndex = entities.indexOf(link.to);
        if (fromIndex < 0 || toIndex < 0) return;
        const from = positions[fromIndex], to = positions[toIndex];
        const gradient = ctx.createLinearGradient(from.x, from.y, to.x, to.y);
        gradient.addColorStop(0, 'rgba(103,169,255,.18)'); gradient.addColorStop(1, 'rgba(66,211,234,.48)');
        ctx.beginPath(); ctx.moveTo(from.x, from.y); ctx.lineTo(to.x, to.y); ctx.strokeStyle = gradient; ctx.lineWidth = 1; ctx.stroke();
        const angle = Math.atan2(to.y - from.y, to.x - from.x), mx = (from.x + to.x) / 2, my = (from.y + to.y) / 2;
        ctx.beginPath(); ctx.moveTo(mx, my); ctx.lineTo(mx - 5 * Math.cos(angle - .45), my - 5 * Math.sin(angle - .45)); ctx.lineTo(mx - 5 * Math.cos(angle + .45), my - 5 * Math.sin(angle + .45)); ctx.closePath();
        ctx.fillStyle = 'rgba(66,211,234,.65)'; ctx.fill();
    });
    positions.forEach((position, index) => {
        const glow = ctx.createRadialGradient(position.x, position.y, 1, position.x, position.y, 16);
        glow.addColorStop(0, `${COLORS[index % COLORS.length]}55`); glow.addColorStop(1, `${COLORS[index % COLORS.length]}00`);
        ctx.beginPath(); ctx.arc(position.x, position.y, 16, 0, Math.PI * 2); ctx.fillStyle = glow; ctx.fill();
        ctx.beginPath(); ctx.arc(position.x, position.y, entities.length > 20 ? 3.2 : 5, 0, Math.PI * 2); ctx.fillStyle = COLORS[index % COLORS.length]; ctx.fill();
        if (entities.length <= 14) {
            ctx.fillStyle = '#9aa8bb'; ctx.font = '9px ui-sans-serif, sans-serif'; ctx.textAlign = 'center';
            ctx.fillText(entities[index], position.x, position.y > cy ? position.y + 18 : position.y - 13);
        }
    });
}

function drawEventDistribution(data) {
    const { ctx, width: w, height: h } = prepareCanvas('event-chart', 260);
    const entries = Object.entries(data.eventTypeCounts || {}).filter(([, count]) => count > 0);
    el('event-empty').classList.toggle('hidden', Boolean(entries.length));
    el('distribution-meta').textContent = `${entries.length} categor${entries.length === 1 ? 'y' : 'ies'}`;
    if (!entries.length) return;
    const pad = { top: 28, right: 16, bottom: 42, left: 38 };
    const cw = w - pad.left - pad.right, ch = h - pad.top - pad.bottom;
    const max = Math.max(...entries.map(([, count]) => count), 1);
    const slot = cw / entries.length, barWidth = Math.min(48, slot * .55);
    ctx.strokeStyle = 'rgba(163,184,214,.09)'; ctx.beginPath(); ctx.moveTo(pad.left, pad.top + ch); ctx.lineTo(pad.left + cw, pad.top + ch); ctx.stroke();
    entries.forEach(([type, count], index) => {
        const height = count / max * ch, x = pad.left + slot * index + (slot - barWidth) / 2, y = pad.top + ch - height;
        const color = TYPE_COLORS[type] || COLORS[index % COLORS.length];
        const gradient = ctx.createLinearGradient(0, y, 0, pad.top + ch);
        gradient.addColorStop(0, color); gradient.addColorStop(1, `${color}35`);
        roundRect(ctx, x, y, barWidth, Math.max(2, height), 5); ctx.fillStyle = gradient; ctx.fill();
        ctx.fillStyle = '#b7c1cf'; ctx.font = '10px ui-monospace, monospace'; ctx.textAlign = 'center'; ctx.fillText(compactNumber(count), x + barWidth / 2, y - 8);
        ctx.fillStyle = '#718097'; ctx.font = '9px ui-sans-serif, sans-serif'; ctx.fillText(prettyName(type), x + barWidth / 2, h - 16);
    });
}

function roundRect(ctx, x, y, width, height, radius) {
    const r = Math.min(radius, width / 2, height / 2);
    ctx.beginPath(); ctx.moveTo(x + r, y); ctx.arcTo(x + width, y, x + width, y + height, r); ctx.arcTo(x + width, y + height, x, y + height, r); ctx.arcTo(x, y + height, x, y, r); ctx.arcTo(x, y, x + width, y, r); ctx.closePath();
}

function redrawCharts() {
    const world = currentWorld();
    if (world) drawEntityNetwork(world);
    if (state.data) {
        drawResourceChart(state.data);
        drawEventDistribution(state.data);
    }
}

/* Small utilities */
function toggleSidebar(open) {
    dom.body.classList.toggle('sidebar-open', open);
    el('menu-button').setAttribute('aria-expanded', String(open));
}

function showToast(title, copy, type = 'success') {
    const toast = document.createElement('div'); toast.className = `toast ${type}`;
    const mark = document.createElement('span'); mark.className = 'toast-mark'; mark.textContent = type === 'error' ? '!' : '✓';
    const body = document.createElement('div');
    const heading = document.createElement('strong'); heading.textContent = title;
    const paragraph = document.createElement('p'); paragraph.textContent = copy;
    body.append(heading, paragraph); toast.append(mark, body); el('toast-region').append(toast);
    setTimeout(() => toast.remove(), 4200);
}

function formatNumber(value) { return Number(value || 0).toLocaleString(); }
function formatDecimal(value, digits = 2) { return Number(value || 0).toLocaleString(undefined, { maximumFractionDigits: digits }); }
function compactNumber(value) { return Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(Number(value || 0)); }
function prettyName(value) { return String(value).replace(/([a-z])([A-Z])/g, '$1 $2').replace(/[_-]/g, ' ').replace(/\b\w/g, char => char.toUpperCase()); }
function shortHash(value, length = 12) { return value ? `${value.slice(0, length)}…${value.slice(-4)}` : '—'; }
function signedDifference(value) { return `${value > 0 ? '+' : ''}${formatNumber(value)}`; }
function niceMaximum(value) {
    if (!value) return 100;
    const magnitude = 10 ** Math.floor(Math.log10(value));
    return Math.ceil(value / magnitude) * magnitude;
}
function relativeTime(timestamp) {
    const seconds = Math.max(0, Math.round((Date.now() - timestamp) / 1000));
    if (seconds < 60) return 'now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
    return `${Math.floor(seconds / 86400)}d`;
}

initialize();
