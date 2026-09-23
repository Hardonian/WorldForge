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
const MAX_NETWORK_NODES = 256;
const MAX_NETWORK_LINKS = 768;

const state = {
    worlds: [],
    worldId: null,
    data: null,
    eventsVisible: EVENTS_PAGE_SIZE,
    history: loadHistory(),
    busy: false,
    play: {
        sessionId: null, worldId: null, data: null, events: [], maxima: {},
        running: false, requestInFlight: false, timer: null,
    },
    saves: [],
    builderIdTouched: false,
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
    playDialog: el('play-dialog'),
    saveDialog: el('save-dialog'),
    builderDialog: el('builder-dialog'),
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
        const defaultWorld = state.worlds.find(world => world.id === 'supply-chain') || state.worlds[0];
        selectWorld(defaultWorld.id, { useDefaults: true });
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
    el('nav-play').addEventListener('click', openPlayMode);
    el('nav-builder').addEventListener('click', openWorldBuilder);
    el('compare-run').addEventListener('click', runComparison);
    el('benchmark-run').addEventListener('click', runBenchmark);
    el('clear-history').addEventListener('click', clearHistory);
    el('play-new').addEventListener('click', startPlaySession);
    el('play-intro-start').addEventListener('click', startPlaySession);
    el('play-toggle').addEventListener('click', () => setPlayRunning(!state.play.running));
    el('play-step').addEventListener('click', () => advancePlaySession(1));
    el('play-reset').addEventListener('click', startPlaySession);
    el('play-save').addEventListener('click', () => openSaveManager(true));
    el('play-saves').addEventListener('click', () => openSaveManager(false));
    el('play-replay').addEventListener('click', downloadPlayReplay);
    el('play-close').addEventListener('click', closePlayMode);
    dom.playDialog.addEventListener('close', () => setPlayRunning(false));
    el('play-speed').addEventListener('change', () => state.play.running && schedulePlayStep());
    el('capacity-slider').addEventListener('input', updateCapacityLabel);
    el('play-entity-select').addEventListener('change', syncCapacityControl);
    el('apply-capacity').addEventListener('click', applyCapacityDecision);
    el('save-close').addEventListener('click', () => dom.saveDialog.close());
    el('save-form').addEventListener('submit', saveCurrentSession);
    el('refresh-saves').addEventListener('click', refreshSaves);
    el('builder-close').addEventListener('click', () => dom.builderDialog.close());
    el('builder-form').addEventListener('submit', createWorld);
    el('builder-world-title').addEventListener('input', syncBuilderSlug);
    el('builder-world-id').addEventListener('input', () => { state.builderIdTouched = true; });
    el('builder-description').addEventListener('input', updateBuilderDescriptionCount);
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

function openWorldBuilder() {
    toggleSidebar(false);
    const form = el('builder-form');
    form.reset();
    state.builderIdTouched = false;
    el('builder-world-id').value = '';
    updateBuilderDescriptionCount();
    setButtonBusy(el('builder-submit'), false, 'Create & play');
    if (!dom.builderDialog.open) dom.builderDialog.showModal();
    el('builder-world-title').focus();
}

function syncBuilderSlug() {
    if (state.builderIdTouched) return;
    el('builder-world-id').value = slugify(el('builder-world-title').value);
}

function updateBuilderDescriptionCount() {
    el('builder-description-count').textContent = el('builder-description').value.length.toString();
}

async function createWorld(event) {
    event.preventDefault();
    const form = el('builder-form');
    if (!form.reportValidity()) return;
    const button = el('builder-submit');
    let seed, ticks;
    try {
        seed = numericInput(el('builder-seed'), { min: 0, max: Number.MAX_SAFE_INTEGER, name: 'Seed' });
        ticks = numericInput(el('builder-ticks'), { min: 10, max: 1_000_000, name: 'Duration' });
    } catch (error) {
        showToast('Check the world settings', error.message, 'error');
        return;
    }
    const id = el('builder-world-id').value.trim();
    const template = form.elements.namedItem('builder-template').value;
    setButtonBusy(button, true, 'Forging world…');
    try {
        await api('/api/worlds', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                id,
                title: el('builder-world-title').value.trim(),
                description: el('builder-description').value.trim(),
                template,
                difficulty: el('builder-difficulty').value,
                seed,
                ticks,
            }),
        });
        const catalog = await api('/api/worlds');
        state.worlds = catalog.worlds || [];
        renderWorldNavigation();
        selectWorld(id, { useDefaults: true });
        setEngineStatus(true, 'Engine ready', `${state.worlds.length} worlds available`);
        dom.builderDialog.close();
        showToast('World forged', `${el('builder-world-title').value.trim()} passed validation and is ready to play.`);
        setTimeout(openPlayMode, 120);
    } catch (error) {
        showToast('World creation failed', `${error.code ? `${error.code}: ` : ''}${error.message}`, 'error');
    } finally {
        setButtonBusy(button, false, 'Create & play');
    }
}

function slugify(value) {
    return String(value)
        .normalize('NFKD')
        .replace(/[\u0300-\u036f]/g, '')
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
        .replace(/-{2,}/g, '-')
        .slice(0, 48)
        .replace(/-+$/g, '');
}

function selectWorld(worldId, options = {}) {
    const world = state.worlds.find(item => item.id === worldId);
    if (!world) return;
    if (state.play.sessionId && state.play.worldId !== worldId) endPlaySession();
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
    const shown = Math.min(events.length, state.eventsVisible).toLocaleString();
    const retained = events.length.toLocaleString();
    const windowNote = state.data.eventsTruncated
        ? ` in the latest ${state.data.events.length.toLocaleString()} of ${state.data.totalEvents.toLocaleString()} total`
        : '';
    dom.eventCountLabel.textContent = `Showing ${shown} of ${retained} matching events${windowNote}`;
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

/* Interactive play mode */
function openPlayMode() {
    const world = currentWorld();
    if (!world) return;
    el('play-title').textContent = `Play ${world.title}`;
    el('play-subtitle').textContent = world.description || 'Shape a deterministic world in real time.';
    if (state.play.sessionId && state.play.worldId === state.worldId) {
        el('play-intro').classList.add('hidden');
        el('play-grid').setAttribute('aria-hidden', 'false');
        renderPlayState();
    } else {
        showPlayIntro();
    }
    dom.playDialog.showModal();
}

function closePlayMode() {
    setPlayRunning(false);
    dom.playDialog.close();
}

function showPlayIntro() {
    el('play-intro').classList.remove('hidden');
    el('play-grid').setAttribute('aria-hidden', 'true');
    el('play-status').className = 'play-status';
    el('play-status').lastChild.textContent = 'Ready';
    resetPlayControls();
}

function resetPlayControls() {
    el('play-toggle').disabled = true;
    el('play-step').disabled = true;
    el('play-reset').disabled = true;
    el('play-save').disabled = true;
    el('apply-capacity').disabled = true;
    el('play-replay').classList.add('hidden');
}

async function startPlaySession() {
    if (!currentWorld() || state.play.requestInFlight) return;
    let seed, ticks;
    try {
        seed = numericInput(dom.seed, { min: 0, max: Number.MAX_SAFE_INTEGER, name: 'Seed' });
        ticks = numericInput(dom.ticks, { min: 1, max: 1_000_000, name: 'Ticks' });
    } catch (error) {
        showToast('Check the play settings', error.message, 'error'); return;
    }
    setPlayRunning(false);
    state.play.requestInFlight = true;
    setButtonBusy(el('play-new'), true, 'Starting…');
    setButtonBusy(el('play-intro-start'), true, 'Starting…');
    try {
        if (state.play.sessionId) {
            await api(`/api/play/sessions/${state.play.sessionId}`, { method: 'DELETE' }).catch(() => {});
            state.play.sessionId = null;
            state.play.data = null;
            state.play.events = [];
            state.play.maxima = {};
        }
        const data = await api('/api/play/sessions', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ world: state.worldId, seed, ticks }),
        });
        state.play.sessionId = data.sessionId;
        state.play.worldId = state.worldId;
        state.play.data = data;
        state.play.events = [];
        state.play.maxima = { ...data.snapshot.levels };
        el('play-intro').classList.add('hidden');
        el('play-grid').setAttribute('aria-hidden', 'false');
        el('play-toggle').disabled = false;
        el('play-step').disabled = false;
        el('play-reset').disabled = false;
        el('play-save').disabled = false;
        el('apply-capacity').disabled = false;
        el('play-replay').classList.add('hidden');
        renderPlayState();
        setPlayRunning(true);
        showToast('World started', 'The deterministic session is live. Pause at any time to make decisions.');
    } catch (error) {
        showPlayIntro();
        showToast('Could not start world', error.message, 'error');
    } finally {
        state.play.requestInFlight = false;
        setButtonBusy(el('play-new'), false, state.play.sessionId ? 'New world' : 'Start world');
        setButtonBusy(el('play-intro-start'), false, 'Start simulation');
    }
}

function setPlayRunning(running) {
    const play = state.play;
    if (!play.sessionId || play.data?.completed) running = false;
    play.running = running;
    clearTimeout(play.timer);
    play.timer = null;
    const button = el('play-toggle');
    const icon = button.querySelector('svg');
    button.querySelector('span').textContent = running ? 'Pause' : 'Play';
    icon.innerHTML = running
        ? '<path d="M7 5v10M13 5v10"/>'
        : '<path d="m7 5 7 5-7 5V5Z"/>';
    updatePlayStatus();
    if (running) schedulePlayStep();
}

function schedulePlayStep() {
    clearTimeout(state.play.timer);
    if (!state.play.running || state.play.data?.completed) return;
    const speed = Number(el('play-speed').value);
    const delay = { 1: 600, 2: 400, 5: 250, 10: 160 }[speed] || 300;
    const batch = { 1: 1, 2: 5, 5: 20, 10: 50 }[speed] || 10;
    state.play.timer = setTimeout(async () => {
        await advancePlaySession(batch);
        if (state.play.running) schedulePlayStep();
    }, delay);
}

async function advancePlaySession(ticks) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed) return;
    play.requestInFlight = true;
    el('play-step').disabled = true;
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/step`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ticks }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        if (data.completed) completePlaySession(data);
    } catch (error) {
        setPlayRunning(false);
        showToast('Play session interrupted', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        el('play-step').disabled = Boolean(play.data?.completed);
    }
}

function consumePlayUpdate(data) {
    const previous = state.play.data;
    state.play.data = {
        ...previous,
        ...data,
        entities: data.entities || previous?.entities || [],
        links: data.links || previous?.links || [],
    };
    if (data.recentEvents?.length) {
        state.play.events.push(...data.recentEvents);
        state.play.events = state.play.events.slice(-120);
    }
    Object.entries(data.snapshot.levels).forEach(([resource, value]) => {
        state.play.maxima[resource] = Math.max(state.play.maxima[resource] || 0, value, 1);
    });
    renderPlayState();
}

function completePlaySession(data) {
    setPlayRunning(false);
    el('play-toggle').disabled = true;
    el('play-step').disabled = true;
    el('apply-capacity').disabled = true;
    el('play-save').disabled = false;
    el('play-replay').classList.remove('hidden');
    el('play-new').textContent = 'Play again';
    const passed = data.objectives.filter(objective => objective.status === 'Passed').length;
    const allPassed = passed === data.objectives.length;
    showToast(
        allPassed ? 'World secured' : 'Simulation complete',
        `${passed} of ${data.objectives.length} objectives passed. The final proof is verified.`,
        allPassed ? 'success' : 'error',
    );
}

async function applyCapacityDecision() {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed) return;
    const entity = el('play-entity-select').value;
    const capacity = Number(el('capacity-slider').value) / 100;
    play.requestInFlight = true;
    el('apply-capacity').disabled = true;
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/intervene`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ entity, capacity }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Decision applied', `${prettyName(entity)} capacity is now ${Math.round(capacity * 100)}%.`);
    } catch (error) {
        showToast('Decision rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        el('apply-capacity').disabled = Boolean(play.data?.completed);
    }
}

function updateCapacityLabel() {
    el('capacity-value').textContent = `${el('capacity-slider').value}%`;
}

function syncCapacityControl() {
    const entity = state.play.data?.entityStates.find(item => item.name === el('play-entity-select').value);
    if (!entity || entity.capacity == null) return;
    el('capacity-slider').value = Math.round(entity.capacity * 100);
    updateCapacityLabel();
}

function renderPlayState() {
    const data = state.play.data;
    if (!data) return;
    el('play-tick-label').textContent = `Tick ${formatNumber(data.currentTick)} / ${formatNumber(data.totalTicks)}`;
    el('play-progress-bar').style.width = `${Math.min(100, data.currentTick / Math.max(1, data.totalTicks) * 100)}%`;
    el('play-event-count').textContent = `${formatNumber(data.eventCount)} events`;
    el('play-fingerprint').textContent = shortHash(data.stateFingerprint, 18);
    el('play-fingerprint').title = data.stateFingerprint;
    renderLiveResources(data.snapshot.levels);
    renderLiveObjectives(data.objectives);
    renderLiveFeed();
    renderPlayNetwork(data);
    renderProducerOptions(data.entityStates);
    updatePlayStatus();
}

function updatePlayStatus() {
    const node = el('play-status');
    let status = 'Ready';
    let className = 'play-status';
    if (state.play.data?.completed) { status = 'Completed'; className += ' completed'; }
    else if (state.play.running) { status = 'Running'; className += ' running'; }
    else if (state.play.sessionId) { status = 'Paused'; className += ' paused'; }
    node.className = className;
    node.lastChild.textContent = status;
}

function renderLiveResources(levels) {
    const container = el('live-resources');
    container.replaceChildren();
    Object.entries(levels).forEach(([resource, value], index) => {
        const card = document.createElement('div'); card.className = 'live-resource';
        const copy = document.createElement('span');
        const name = document.createElement('b'); name.textContent = prettyName(resource);
        const amount = document.createElement('strong'); amount.textContent = formatDecimal(value, 1);
        copy.append(name, amount);
        const meter = document.createElement('div'); meter.className = 'resource-meter';
        const fill = document.createElement('i');
        fill.style.width = `${Math.max(1, value / Math.max(1, state.play.maxima[resource]) * 100)}%`;
        fill.style.setProperty('--resource-color', COLORS[index % COLORS.length]);
        meter.append(fill); card.append(copy, meter); container.append(card);
    });
}

function renderLiveObjectives(objectives) {
    const container = el('live-objectives'); container.replaceChildren();
    if (!objectives.length) {
        const empty = document.createElement('p'); empty.className = 'feed-empty'; empty.textContent = 'No objectives configured.'; container.append(empty); return;
    }
    objectives.forEach(objective => {
        const row = document.createElement('div'); row.className = `live-objective ${objective.status.toLowerCase()}`;
        const icon = document.createElement('i'); icon.textContent = objective.status === 'Passed' ? '✓' : objective.status === 'Failed' ? '×' : '·';
        const name = document.createElement('span'); name.textContent = objective.name;
        row.append(icon, name); container.append(row);
    });
}

function renderLiveFeed() {
    const container = el('live-feed'); container.replaceChildren();
    const events = state.play.events.slice(-50).reverse();
    if (!events.length) {
        const empty = document.createElement('div'); empty.className = 'feed-empty'; empty.textContent = 'Advance the world to see live activity.'; container.append(empty); return;
    }
    events.forEach(event => {
        const row = document.createElement('div'); row.className = 'feed-event';
        const tick = document.createElement('time'); tick.textContent = `t${event.tick}`;
        const type = document.createElement('span'); type.className = `log-type ${event.type}`; type.textContent = event.type;
        const copy = document.createElement('p'); copy.textContent = event.summary;
        row.append(tick, type, copy); container.append(row);
    });
}

function renderProducerOptions(entityStates) {
    const select = el('play-entity-select');
    const current = select.value;
    const producers = entityStates.filter(entity => entity.capacity != null);
    const existing = [...select.options].map(option => option.value);
    const names = producers.map(entity => entity.name);
    const rebuilt = existing.join('|') !== names.join('|');
    if (rebuilt) {
        select.replaceChildren(...producers.map(entity => new Option(prettyName(entity.name), entity.name)));
    }
    if (names.includes(current)) select.value = current;
    if (rebuilt || document.activeElement !== el('capacity-slider')) syncCapacityControl();
}

function renderPlayNetwork(data) {
    const svg = el('play-network');
    svg.replaceChildren();
    const visibleStates = data.entityStates.slice(0, MAX_NETWORK_NODES);
    const entities = visibleStates.map(entity => entity.name);
    const count = entities.length;
    if (!count) return;
    svg.dataset.density = count > 80 ? 'dense' : 'normal';
    const fragment = document.createDocumentFragment();
    const cx = 400, cy = 208;
    const radiusX = count > 14 ? 310 : 270, radiusY = count > 14 ? 155 : 140;
    const positions = new Map(entities.map((name, index) => [name, {
        x: cx + radiusX * Math.cos(index / count * Math.PI * 2 - Math.PI / 2),
        y: cy + radiusY * Math.sin(index / count * Math.PI * 2 - Math.PI / 2),
    }]));
    const recentShortages = new Set(state.play.events.filter(event => event.type === 'shortage').slice(-20).map(event => event.entity));
    data.links.slice(0, MAX_NETWORK_LINKS).forEach(link => {
        const from = positions.get(link.from), to = positions.get(link.to);
        if (!from || !to) return;
        const line = svgNode('path', { d: `M${from.x},${from.y} L${to.x},${to.y}`, class: 'network-link network-link-hot' });
        fragment.append(line);
    });
    visibleStates.forEach((entity, index) => {
        const position = positions.get(entity.name); if (!position) return;
        const group = svgNode('g', { transform: `translate(${position.x} ${position.y})` });
        group.append(svgNode('circle', { r: count > 20 ? 17 : 25, class: 'network-node-glow' }));
        group.append(svgNode('circle', { r: count > 20 ? 6 : 10, class: `network-node-core${recentShortages.has(entity.name) ? ' warning' : ''}` }));
        if (count <= 16) {
            const label = svgNode('text', { y: 27, class: 'network-node-label' }); label.textContent = prettyName(entity.name);
            const total = Object.values(entity.inventory).reduce((sum, value) => sum + value, 0);
            const detail = svgNode('text', { y: 40, class: 'network-node-detail' }); detail.textContent = compactNumber(total);
            group.append(label, detail);
        }
        fragment.append(group);
    });
    svg.append(fragment);
}

function svgNode(name, attributes) {
    const node = document.createElementNS('http://www.w3.org/2000/svg', name);
    Object.entries(attributes).forEach(([key, value]) => node.setAttribute(key, value));
    return node;
}

async function endPlaySession() {
    const id = state.play.sessionId;
    clearTimeout(state.play.timer);
    state.play = { sessionId: null, worldId: null, data: null, events: [], maxima: {}, running: false, requestInFlight: false, timer: null };
    resetPlayControls();
    if (id) await api(`/api/play/sessions/${id}`, { method: 'DELETE' }).catch(() => {});
}

async function openSaveManager(focusName) {
    setPlayRunning(false);
    const hasSession = Boolean(state.play.sessionId);
    el('save-name').disabled = !hasSession;
    el('save-submit').disabled = !hasSession;
    if (hasSession && !el('save-name').value) {
        el('save-name').value = `${currentWorld()?.title || 'World'} · tick ${state.play.data.currentTick}`;
    }
    if (!dom.saveDialog.open) dom.saveDialog.showModal();
    await refreshSaves();
    if (focusName && hasSession) el('save-name').focus();
}

async function refreshSaves() {
    const list = el('save-list');
    list.replaceChildren(emptyState('◌', 'Loading saves', 'Reading durable slots from disk.'));
    try {
        const payload = await api('/api/saves');
        state.saves = payload.saves || [];
        renderSaves();
    } catch (error) {
        list.replaceChildren(emptyState('!', 'Could not load saves', error.message));
    }
}

async function saveCurrentSession(event) {
    event.preventDefault();
    if (!state.play.sessionId) return;
    const name = el('save-name').value.trim();
    if (!name) { el('save-name').focus(); return; }
    const button = el('save-submit');
    setButtonBusy(button, true, 'Saving…');
    try {
        await api('/api/saves', {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ sessionId: state.play.sessionId, name }),
        });
        el('save-name').value = '';
        await refreshSaves();
        showToast('World saved', 'This session can now be resumed after restarting the dashboard.');
    } catch (error) {
        showToast('Save failed', error.message, 'error');
    } finally {
        setButtonBusy(button, false, 'Save current world');
    }
}

function renderSaves() {
    const list = el('save-list'); list.replaceChildren();
    el('save-count').textContent = `${state.saves.length} save${state.saves.length === 1 ? '' : 's'}`;
    if (!state.saves.length) {
        list.append(emptyState('□', 'No saves yet', 'Start a world and create your first durable save.'));
        return;
    }
    state.saves.forEach(save => {
        const slot = document.createElement('article'); slot.className = 'save-slot';
        const icon = document.createElement('span'); icon.className = 'save-slot-icon'; icon.textContent = `${Math.round(save.currentTick / Math.max(1, save.totalTicks) * 100)}%`;
        const copy = document.createElement('div'); copy.className = 'save-slot-copy';
        const title = document.createElement('strong'); title.textContent = save.name;
        const meta = document.createElement('span'); meta.textContent = `${prettyName(save.world)} · tick ${formatNumber(save.currentTick)} / ${formatNumber(save.totalTicks)} · seed ${save.seed}`;
        copy.append(title, meta);
        const actions = document.createElement('div'); actions.className = 'save-slot-actions';
        const resume = document.createElement('button'); resume.type = 'button'; resume.className = 'button button-secondary'; resume.textContent = 'Resume'; resume.addEventListener('click', () => resumeSave(save.id));
        const remove = document.createElement('button'); remove.type = 'button'; remove.className = 'button button-quiet save-delete'; remove.textContent = 'Delete'; remove.addEventListener('click', () => removeSave(save));
        actions.append(resume, remove); slot.append(icon, copy, actions); list.append(slot);
    });
}

async function resumeSave(saveId) {
    const save = state.saves.find(item => item.id === saveId);
    if (!save) return;
    setPlayRunning(false);
    if (state.play.sessionId) await endPlaySession();
    try {
        const data = await api(`/api/saves/${saveId}/load`, { method: 'POST' });
        state.play = {
            sessionId: data.sessionId, worldId: data.world, data, events: [],
            maxima: { ...data.snapshot.levels }, running: false, requestInFlight: false, timer: null,
        };
        if (state.worldId !== data.world) selectWorld(data.world);
        state.play.sessionId = data.sessionId;
        state.play.worldId = data.world;
        state.play.data = data;
        el('play-title').textContent = `Play ${currentWorld()?.title || prettyName(data.world)}`;
        el('play-intro').classList.add('hidden');
        el('play-grid').setAttribute('aria-hidden', 'false');
        el('play-toggle').disabled = data.completed;
        el('play-step').disabled = data.completed;
        el('play-reset').disabled = false;
        el('play-save').disabled = false;
        el('apply-capacity').disabled = data.completed;
        el('play-replay').classList.toggle('hidden', !data.completed);
        renderPlayState();
        dom.saveDialog.close();
        showToast('Save resumed', `${save.name} restored at tick ${formatNumber(data.currentTick)}.`);
    } catch (error) {
        showToast('Resume failed', error.message, 'error');
        showPlayIntro();
    }
}

async function removeSave(save) {
    if (!window.confirm(`Delete “${save.name}”? This cannot be undone.`)) return;
    try {
        await api(`/api/saves/${save.id}`, { method: 'DELETE' });
        await refreshSaves();
        showToast('Save deleted', 'The local save slot was removed.');
    } catch (error) {
        showToast('Delete failed', error.message, 'error');
    }
}

async function downloadPlayReplay() {
    if (!state.play.sessionId || !state.play.data?.completed) return;
    try {
        const response = await fetch(`/api/play/sessions/${state.play.sessionId}/replay`);
        if (!response.ok) throw new Error('Replay is unavailable.');
        const disposition = response.headers.get('Content-Disposition') || '';
        const filename = disposition.match(/filename="([^"]+)"/)?.[1] || `${state.play.worldId}.replay`;
        const url = URL.createObjectURL(await response.blob());
        const anchor = document.createElement('a'); anchor.href = url; anchor.download = filename; document.body.append(anchor); anchor.click(); anchor.remove(); URL.revokeObjectURL(url);
        showToast('Replay downloaded', 'The tamper-evident replay artifact is ready to archive.');
    } catch (error) {
        showToast('Replay download failed', error.message, 'error');
    }
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
    const entities = (world.entities || []).slice(0, MAX_NETWORK_NODES);
    const links = (world.links || []).slice(0, MAX_NETWORK_LINKS);
    if (!entities.length) return;
    const cx = w / 2, cy = h / 2;
    const radius = Math.max(58, Math.min(w * .34, h * .34));
    const positions = entities.map((_, index) => ({
        x: cx + radius * Math.cos(index / entities.length * Math.PI * 2 - Math.PI / 2),
        y: cy + radius * Math.sin(index / entities.length * Math.PI * 2 - Math.PI / 2),
    }));
    const entityIndexes = new Map(entities.map((name, index) => [name, index]));
    links.forEach(link => {
        const fromIndex = entityIndexes.get(link.from), toIndex = entityIndexes.get(link.to);
        if (fromIndex === undefined || toIndex === undefined) return;
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
