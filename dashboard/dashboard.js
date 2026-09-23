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
    'coastal-resilience': 'CR',
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
    dom.playDialog.addEventListener('close', () => { setPlayRunning(false); WorldForgeCG.stop(); });
    el('play-speed').addEventListener('change', () => state.play.running && schedulePlayStep());
    el('capacity-slider').addEventListener('input', updateCapacityLabel);
    el('play-entity-select').addEventListener('change', syncCapacityControl);
    el('apply-capacity').addEventListener('click', applyCapacityDecision);
    WorldForgeCG.bindControls();
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
    renderOperationalInsights(data);
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
    resetOperationalInsights();
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
        detail.textContent = objectiveDetail(objective);
        const progress = document.createElement('span');
        progress.className = 'objective-progress';
        progress.setAttribute('role', 'progressbar');
        progress.setAttribute('aria-valuemin', '0');
        progress.setAttribute('aria-valuemax', '100');
        const percentage = Math.round(Math.max(0, Math.min(1, Number(objective.progress) || 0)) * 100);
        progress.setAttribute('aria-valuenow', percentage.toString());
        progress.setAttribute('aria-label', `${objective.name}: ${percentage}% progress`);
        const fill = document.createElement('i');
        fill.style.width = `${percentage}%`;
        progress.append(fill);
        copy.append(name, detail, progress);
        card.append(icon, copy);
        container.append(card);
    });
}

function objectiveDetail(objective) {
    const current = formatDecimal(objective.current, 1);
    const target = formatDecimal(objective.target, 1);
    switch (objective.kind) {
        case 'maintain_inventory': return `Minimum ${current} · floor ${target}`;
        case 'avoid_shortage': return `${formatNumber(objective.current)} shortage events · target 0`;
        case 'reach_production_target': return `Produced ${current} of ${target}`;
        case 'reach_inventory_target': return `Peak ${current} of ${target}`;
        case 'survive_until_tick': return `Tick ${formatNumber(objective.current)} of ${formatNumber(objective.target)}`;
        default: return objective.status;
    }
}

function renderOperationalInsights(data) {
    const objectives = data.objectives || [];
    const metrics = data.resourceMetrics || [];
    const objectiveScore = objectives.length
        ? objectives.reduce((sum, objective) => sum + (objective.status === 'Passed' ? 1 : objective.status === 'Pending' ? .5 : 0), 0) / objectives.length
        : 1;
    const shortagePressure = Math.min(1, Number(data.shortageEvents || 0) / Math.max(1, Number(data.ticks || 1)));
    const score = Math.round((objectiveScore * .6 + (1 - shortagePressure) * .4) * 100);
    const grade = score >= 90 ? 'Resilient' : score >= 70 ? 'Stable' : score >= 50 ? 'Strained' : 'Critical';
    const ring = el('resilience-ring');
    ring.style.setProperty('--score', score);
    ring.setAttribute('aria-label', `Resilience score ${score} out of 100, ${grade}`);
    el('resilience-score').textContent = score.toString();
    el('resilience-grade').textContent = grade;
    el('resilience-label').textContent = `${objectives.filter(item => item.status === 'Passed').length}/${objectives.length || 0} goals · ${formatDecimal(shortagePressure * 100, 1)}% pressure`;

    const stressed = metrics
        .filter(metric => Number(metric.initial) > 0)
        .map(metric => ({ ...metric, drawdown: Math.max(0, (Number(metric.initial) - Number(metric.minimum)) / Number(metric.initial)) }))
        .sort((a, b) => b.drawdown - a.drawdown)[0];
    const growth = [...metrics].sort((a, b) => Number(b.netChange) - Number(a.netChange))[0];
    const recovery = [...metrics]
        .map(metric => ({ ...metric, recovery: Number(metric.finalLevel) - Number(metric.minimum) }))
        .sort((a, b) => b.recovery - a.recovery)[0];
    const activity = Number(data.totalEvents || 0) / Math.max(1, Number(data.ticks || 1));

    const cards = el('insight-cards');
    cards.replaceChildren(
        insightMetric('Most stressed', stressed ? prettyName(stressed.resource) : 'No depletion', stressed ? `${formatDecimal(stressed.drawdown * 100, 1)}% drawdown · low at t${formatNumber(stressed.minimumTick)}` : 'Inventories held their floor', 'stress'),
        insightMetric('Strongest gain', growth ? prettyName(growth.resource) : 'No resources', growth ? `${signedDecimal(growth.netChange)} net units` : 'No resource telemetry', 'growth'),
        insightMetric('Best recovery', recovery ? prettyName(recovery.resource) : 'No recovery', recovery ? `${signedDecimal(recovery.recovery)} from low to finish` : 'No resource telemetry', 'recovery'),
        insightMetric('World activity', formatDecimal(activity, 2), `${formatNumber(data.totalEvents)} events across ${formatNumber(data.ticks)} ticks`, 'activity'),
    );

    const ledger = el('resource-ledger');
    ledger.replaceChildren();
    if (!metrics.length) {
        const empty = document.createElement('div');
        empty.className = 'ledger-empty';
        empty.textContent = 'This run did not expose tracked resources.';
        ledger.append(empty);
        return;
    }
    metrics.forEach(metric => {
        const row = document.createElement('div'); row.className = 'resource-ledger-row';
        const resource = document.createElement('strong'); resource.textContent = prettyName(metric.resource);
        const movement = document.createElement('span'); movement.textContent = `${formatDecimal(metric.initial, 1)} → ${formatDecimal(metric.finalLevel, 1)}`;
        const range = document.createElement('span'); range.textContent = `${formatDecimal(metric.minimum, 1)}–${formatDecimal(metric.maximum, 1)}`;
        range.title = `Minimum at tick ${metric.minimumTick}; maximum at tick ${metric.maximumTick}`;
        const delta = document.createElement('span');
        const net = Number(metric.netChange || 0);
        delta.className = `ledger-delta ${net > 0 ? 'positive' : net < 0 ? 'negative' : 'neutral'}`;
        delta.textContent = signedDecimal(net);
        row.append(resource, movement, range, delta);
        ledger.append(row);
    });
}

function insightMetric(label, value, detail, tone) {
    const card = document.createElement('article'); card.className = `insight-metric ${tone}`;
    const labelNode = document.createElement('span'); labelNode.textContent = label;
    const valueNode = document.createElement('strong'); valueNode.textContent = value;
    const detailNode = document.createElement('small'); detailNode.textContent = detail;
    card.append(labelNode, valueNode, detailNode);
    return card;
}

function resetOperationalInsights() {
    const ring = el('resilience-ring');
    ring.style.setProperty('--score', 0);
    ring.setAttribute('aria-label', 'Resilience score not yet calculated');
    el('resilience-score').textContent = '—';
    el('resilience-grade').textContent = 'Not evaluated';
    el('resilience-label').textContent = 'Awaiting run';
    el('insight-cards').replaceChildren(emptyState('⌁', '', 'Run a world to identify stress, growth, recovery, and activity.', true));
    const empty = document.createElement('div'); empty.className = 'ledger-empty'; empty.textContent = 'Exact per-tick extrema will appear after a run.';
    el('resource-ledger').replaceChildren(empty);
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
    WorldForgeCG.start();
    dom.playDialog.showModal();
}

function closePlayMode() {
    setPlayRunning(false);
    WorldForgeCG.stop();
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
        WorldForgeCG.onEvents(data.recentEvents);
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
    WorldForgeCG.setSelectedEntity(el('play-entity-select').value);
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
    WorldForgeCG.setData(data);
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
        const copy = document.createElement('span'); copy.className = 'live-objective-copy';
        const name = document.createElement('b'); name.textContent = objective.name;
        const detail = document.createElement('small'); detail.textContent = objectiveDetail(objective);
        const meter = document.createElement('span'); meter.className = 'live-objective-meter';
        const fill = document.createElement('span'); fill.style.width = `${Math.round(Math.max(0, Math.min(1, Number(objective.progress) || 0)) * 100)}%`;
        meter.append(fill); copy.append(name, detail, meter);
        row.append(icon, copy); container.append(row);
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
function signedDecimal(value, digits = 1) { return `${Number(value) > 0 ? '+' : ''}${formatDecimal(value, digits)}`; }
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

/* ============================================================================
 * World Forge — High-Performance Tactical CG Graphics & Game Animation Engine
 * ============================================================================ */

const RESOURCE_NEON = {
    ore: '#f1b96b', steel: '#9b8cff', goods: '#46d29a', energy: '#42d3ea',
    power: '#42d3ea', wind: '#67a9ff', water: '#38bdf8', seawater: '#0ea5e9',
    food: '#a3e635', biomass: '#84cc16', feedstock: '#fb923c', chemicals: '#f97316',
    medicine: '#f43f5e', care: '#ec4899', sunlight: '#facc15', prey: '#34d399',
    herbivores: '#34d399', carnivores: '#f87171', residents: '#e879f9', alloy: '#c084fc',
};

function getResourceNeonColor(resource) {
    if (!resource) return '#67a9ff';
    const key = String(resource).toLowerCase();
    return RESOURCE_NEON[key] || COLORS[Math.abs(key.split('').reduce((a, c) => a + c.charCodeAt(0), 0)) % COLORS.length];
}

function detectArchetype(entity) {
    const type = (entity.entity_type || '').toLowerCase();
    const name = (entity.name || '').toLowerCase();
    if (type.includes('renewable') || type.includes('power') || name.includes('wind') || name.includes('solar') || name.includes('sunlight')) return 'power';
    if (type.includes('producer') || type.includes('extractor') || name.includes('mine') || name.includes('quarry')) return 'extractor';
    if (type.includes('processor') || type.includes('refiner') || name.includes('mill') || name.includes('chemical') || name.includes('desal')) return 'processor';
    if (type.includes('manufacturer') || name.includes('factory')) return 'manufacturer';
    if (type.includes('distributor') || name.includes('warehouse') || name.includes('depot') || name.includes('port')) return 'depot';
    if (type.includes('consumer') || name.includes('city') || name.includes('market') || name.includes('population') || name.includes('resident')) return 'habitat';
    if (type.includes('food') || name.includes('farm') || name.includes('vegetation') || name.includes('prey') || name.includes('carnivore')) return 'bio';
    if (type.includes('water') || name.includes('water')) return 'water';
    if (type.includes('care') || name.includes('medical') || name.includes('care')) return 'care';
    return 'generic';
}

const ARCHETYPE_META = {
    power: { color: '#42d3ea', label: 'ENERGY CORE' },
    extractor: { color: '#f1b96b', label: 'EXTRACTOR' },
    processor: { color: '#9b8cff', label: 'REFINERY' },
    manufacturer: { color: '#46d29a', label: 'ASSEMBLY' },
    depot: { color: '#38bdf8', label: 'LOGISTICS' },
    habitat: { color: '#f59e0b', label: 'HABITAT' },
    bio: { color: '#a3e635', label: 'ORGANIC' },
    water: { color: '#06b6d4', label: 'HYDRO' },
    care: { color: '#f43f5e', label: 'MEDICAL' },
    generic: { color: '#67a9ff', label: 'FACILITY' },
};

const WorldForgeCG = {
    canvas: null,
    ctx: null,
    hud: null,
    active: false,
    viewMode: 'cg',
    vfxEnabled: true,
    rafId: null,
    lastTime: 0,
    radarAngle: 0,

    data: null,
    nodes: new Map(),
    links: [],
    particles: [],
    shockwaves: [],
    floaties: [],
    ambientDust: [],

    camera: {
        x: 0, y: 0, zoom: 1,
        targetX: 0, targetY: 0, targetZoom: 1,
        isDragging: false,
        dragStartX: 0, dragStartY: 0,
        camStartX: 0, camStartY: 0,
        hasInteracted: false,
    },

    hoveredNode: null,
    selectedNodeName: null,
    lastPointerX: 0,
    lastPointerY: 0,

    init() {
        if (this.canvas) return;
        this.canvas = el('play-cg-canvas');
        if (!this.canvas) return;
        this.ctx = this.canvas.getContext('2d');
        this.hud = el('cg-hud-card');

        // Initialize ambient cyber dust particles
        this.ambientDust = Array.from({ length: 45 }, () => ({
            x: (Math.random() - 0.5) * 1200,
            y: (Math.random() - 0.5) * 800,
            r: Math.random() * 1.5 + 0.5,
            speedX: (Math.random() - 0.5) * 0.18,
            speedY: (Math.random() - 0.5) * 0.18,
            alpha: Math.random() * 0.4 + 0.1,
            color: Math.random() > 0.5 ? '#67a9ff' : '#42d3ea',
        }));

        this.bindEvents();
    },

    bindControls() {
        this.init();
        el('btn-view-cg')?.addEventListener('click', () => this.setViewMode('cg'));
        el('btn-view-schematic')?.addEventListener('click', () => this.setViewMode('schematic'));
        el('cg-btn-zoom-in')?.addEventListener('click', () => this.zoomBy(1.28));
        el('cg-btn-zoom-out')?.addEventListener('click', () => this.zoomBy(0.78));
        el('cg-btn-reset-cam')?.addEventListener('click', () => this.fitView(true));
        el('cg-btn-fx-toggle')?.addEventListener('click', () => this.toggleVfx());
    },

    bindEvents() {
        const c = this.canvas;
        if (!c) return;

        // Pointer / mouse drag
        c.addEventListener('pointerdown', e => {
            if (e.button !== 0) return;
            c.setPointerCapture(e.pointerId);
            this.camera.isDragging = true;
            this.camera.dragStartX = e.clientX;
            this.camera.dragStartY = e.clientY;
            this.camera.camStartX = this.camera.targetX;
            this.camera.camStartY = this.camera.targetY;
        });

        window.addEventListener('pointermove', e => {
            if (this.camera.isDragging) {
                const dx = e.clientX - this.camera.dragStartX;
                const dy = e.clientY - this.camera.dragStartY;
                this.camera.targetX = this.camera.camStartX + dx;
                this.camera.targetY = this.camera.camStartY + dy;
                this.camera.hasInteracted = true;
            } else if (this.active && this.viewMode === 'cg') {
                const rect = c.getBoundingClientRect();
                this.lastPointerX = e.clientX - rect.left;
                this.lastPointerY = e.clientY - rect.top;
                this.checkHover(this.lastPointerX, this.lastPointerY);
            }
        });

        window.addEventListener('pointerup', e => {
            if (this.camera.isDragging) {
                this.camera.isDragging = false;
                try { c.releasePointerCapture(e.pointerId); } catch (_) {}
            }
        });

        // Click selection
        c.addEventListener('click', e => {
            const rect = c.getBoundingClientRect();
            const px = e.clientX - rect.left;
            const py = e.clientY - rect.top;
            const hit = this.getNodeAt(px, py);
            if (hit) {
                this.setSelectedEntity(hit.name);
                const select = el('play-entity-select');
                if (select) {
                    select.value = hit.name;
                    syncCapacityControl();
                }
                this.shockwaves.push({
                    x: hit.x, y: hit.y, r: hit.radius, maxR: hit.radius + 36,
                    color: '#f1b96b', alpha: 1, width: 2.2,
                });
            }
        });

        // Mouse wheel zoom to cursor
        c.addEventListener('wheel', e => {
            e.preventDefault();
            const rect = c.getBoundingClientRect();
            const px = e.clientX - rect.left;
            const py = e.clientY - rect.top;
            const factor = e.deltaY < 0 ? 1.15 : 0.87;
            this.zoomAt(px, py, factor);
        }, { passive: false });

        window.addEventListener('resize', () => {
            if (this.active) this.fitView(false);
        });
    },

    setViewMode(mode) {
        this.viewMode = mode;
        const btnCg = el('btn-view-cg');
        const btnSchem = el('btn-view-schematic');
        const canvas = el('play-cg-canvas');
        const svg = el('play-network');
        const toolbar = el('cg-toolbar');
        const hud = el('cg-hud-card');

        if (mode === 'cg') {
            btnCg?.classList.add('active');
            btnSchem?.classList.remove('active');
            canvas?.classList.remove('hidden');
            svg?.classList.add('hidden');
            toolbar?.classList.remove('hidden');
            this.start();
        } else {
            btnCg?.classList.remove('active');
            btnSchem?.classList.add('active');
            canvas?.classList.add('hidden');
            svg?.classList.remove('hidden');
            toolbar?.classList.add('hidden');
            hud?.classList.add('hidden');
            this.stop();
        }
    },

    toggleVfx() {
        this.vfxEnabled = !this.vfxEnabled;
        const btn = el('cg-btn-fx-toggle');
        btn?.classList.toggle('active', this.vfxEnabled);
        showToast('VFX settings', `Visual effects & particles ${this.vfxEnabled ? 'enabled' : 'disabled'}.`);
    },

    zoomBy(factor) {
        const w = this.canvas ? this.canvas.clientWidth / 2 : 400;
        const h = this.canvas ? this.canvas.clientHeight / 2 : 210;
        this.zoomAt(w, h, factor);
    },

    zoomAt(px, py, factor) {
        if (!this.canvas) return;
        const rect = this.canvas.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const cx = rect.width / 2;
        const cy = rect.height / 2;

        const worldX = (px - cx - this.camera.targetX) / this.camera.targetZoom;
        const worldY = (py - cy - this.camera.targetY) / this.camera.targetZoom;

        const newZoom = Math.max(0.38, Math.min(3.2, this.camera.targetZoom * factor));
        this.camera.targetZoom = newZoom;
        this.camera.targetX = px - cx - worldX * newZoom;
        this.camera.targetY = py - cy - worldY * newZoom;
        this.camera.hasInteracted = true;
    },

    fitView(force = false) {
        if (!this.canvas || (!force && this.camera.hasInteracted)) return;
        const count = this.nodes.size;
        if (!count) return;

        let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
        this.nodes.forEach(node => {
            minX = Math.min(minX, node.targetX - node.radius);
            maxX = Math.max(maxX, node.targetX + node.radius);
            minY = Math.min(minY, node.targetY - node.radius);
            maxY = Math.max(maxY, node.targetY + node.radius);
        });

        const w = this.canvas.clientWidth || 800;
        const h = this.canvas.clientHeight || 420;
        const boxW = Math.max(100, maxX - minX + 170);
        const boxH = Math.max(100, maxY - minY + 150);

        const zoom = Math.max(0.45, Math.min(1.35, Math.min(w / boxW, h / boxH)));
        const midX = (minX + maxX) / 2;
        const midY = (minY + maxY) / 2;

        this.camera.targetZoom = zoom;
        this.camera.targetX = -midX * zoom;
        this.camera.targetY = -midY * zoom;

        if (force) {
            this.camera.x = this.camera.targetX;
            this.camera.y = this.camera.targetY;
            this.camera.zoom = this.camera.targetZoom;
            this.camera.hasInteracted = false;
        }
    },

    start() {
        this.init();
        if (this.active) return;
        this.active = true;
        this.lastTime = performance.now();
        const tick = now => {
            if (!this.active) return;
            this.render(now);
            this.rafId = requestAnimationFrame(tick);
        };
        this.rafId = requestAnimationFrame(tick);
    },

    stop() {
        this.active = false;
        if (this.rafId) {
            cancelAnimationFrame(this.rafId);
            this.rafId = null;
        }
        if (this.hud) this.hud.classList.add('hidden');
    },

    setData(data) {
        this.data = data;
        this.init();
        if (!data || !data.entityStates) return;

        const entities = data.entityStates.slice(0, MAX_NETWORK_NODES);
        const rawEntities = data.entities || [];
        const links = data.links || [];

        // Build entity map & update states
        const currentNames = new Set(entities.map(e => e.name));
        // Remove old nodes
        for (const [name] of this.nodes) {
            if (!currentNames.has(name)) this.nodes.delete(name);
        }

        // Layout computation if node set changed
        const needsLayout = entities.some(e => !this.nodes.has(e.name));
        const positions = needsLayout ? this.computeLayout(entities, links) : null;

        entities.forEach(entityState => {
            const raw = rawEntities.find(e => e.name === entityState.name) || {};
            const archetype = detectArchetype({ ...raw, ...entityState });
            const archetypeTheme = ARCHETYPE_META[archetype] || ARCHETYPE_META.generic;
            let node = this.nodes.get(entityState.name);

            if (!node) {
                const pos = positions?.get(entityState.name) || { x: 0, y: 0 };
                node = {
                    name: entityState.name,
                    entityType: raw.entity_type || entityState.entity_type || 'facility',
                    region: raw.region || entityState.region || 'zone',
                    archetype,
                    theme: archetypeTheme,
                    x: pos.x,
                    y: pos.y,
                    targetX: pos.x,
                    targetY: pos.y,
                    radius: entities.length > 16 ? 24 : 32,
                    inventory: entityState.inventory || {},
                    capacity: entityState.capacity != null ? entityState.capacity : 1.0,
                    production: raw.production || entityState.production || null,
                    pulse: 0,
                    warningPulse: 0,
                    sparkTimer: 0,
                };
                this.nodes.set(entityState.name, node);
            } else {
                node.inventory = entityState.inventory || {};
                node.capacity = entityState.capacity != null ? entityState.capacity : node.capacity;
                if (positions?.has(entityState.name)) {
                    const pos = positions.get(entityState.name);
                    node.targetX = pos.x;
                    node.targetY = pos.y;
                }
            }
        });

        // Update links & particles
        this.links = links.slice(0, MAX_NETWORK_LINKS).map(link => {
            const from = this.nodes.get(link.from);
            const to = this.nodes.get(link.to);
            return {
                from: link.from,
                to: link.to,
                resource: link.resource,
                maxPerTick: link.max_per_tick || 1,
                color: getResourceNeonColor(link.resource),
                valid: Boolean(from && to),
            };
        }).filter(link => link.valid);

        // Adjust particle pool based on links
        if (this.vfxEnabled && this.links.length > 0) {
            const targetParticles = Math.min(100, Math.max(16, this.links.length * 6));
            while (this.particles.length < targetParticles) {
                const link = this.links[Math.floor(Math.random() * this.links.length)];
                this.particles.push({
                    link,
                    t: Math.random(),
                    speed: 0.0035 + Math.random() * 0.004,
                    r: Math.random() * 1.5 + 2.4,
                    color: link.color,
                });
            }
        }

        if (needsLayout) {
            this.fitView(true);
        }
    },

    computeLayout(entities, links) {
        const count = entities.length;
        const positions = new Map();
        if (!count) return positions;

        // Build simple adjacency for tier estimation
        const incoming = new Map(entities.map(e => [e.name, []]));
        const outgoing = new Map(entities.map(e => [e.name, []]));
        links.forEach(link => {
            outgoing.get(link.from)?.push(link.to);
            incoming.get(link.to)?.push(link.from);
        });

        // Compute flow ranks: producers = rank 0, downstream = rank + 1
        const ranks = new Map();
        entities.forEach(e => {
            const inc = incoming.get(e.name) || [];
            if (!inc.length) ranks.set(e.name, 0);
        });

        // Propagate ranks
        for (let iter = 0; iter < 6; iter++) {
            entities.forEach(e => {
                const inc = incoming.get(e.name) || [];
                if (inc.length) {
                    const maxInc = Math.max(...inc.map(p => ranks.get(p) ?? 0));
                    ranks.set(e.name, Math.min(4, maxInc + 1));
                }
            });
        }

        // Check if topological ranking gave good variance
        const rankGroups = new Map();
        ranks.forEach((rank, name) => {
            if (!rankGroups.has(rank)) rankGroups.set(rank, []);
            rankGroups.get(rank).push(name);
        });

        if (rankGroups.size > 1 && count <= 24) {
            // Tiered flow layout (left to right)
            const sortedRanks = [...rankGroups.keys()].sort((a, b) => a - b);
            const totalCols = sortedRanks.length;
            const widthSpan = count > 10 ? 640 : 540;
            const xStep = totalCols > 1 ? widthSpan / (totalCols - 1) : 0;
            const startX = -widthSpan / 2;

            sortedRanks.forEach((rank, colIdx) => {
                const colNodes = rankGroups.get(rank);
                const colCount = colNodes.length;
                const heightSpan = Math.min(320, Math.max(90, (colCount - 1) * 95));
                const yStep = colCount > 1 ? heightSpan / (colCount - 1) : 0;
                const startY = -heightSpan / 2;

                colNodes.forEach((name, rowIdx) => {
                    positions.set(name, {
                        x: Math.round(startX + colIdx * xStep),
                        y: Math.round(startY + rowIdx * yStep),
                    });
                });
            });
        } else {
            // Organic tactical ellipse layout
            const rx = count > 14 ? 310 : 260;
            const ry = count > 14 ? 160 : 135;
            entities.forEach((e, idx) => {
                const angle = (idx / count) * Math.PI * 2 - Math.PI / 2;
                positions.set(e.name, {
                    x: Math.round(rx * Math.cos(angle)),
                    y: Math.round(ry * Math.sin(angle)),
                });
            });
        }

        return positions;
    },

    onEvents(events) {
        if (!events || !events.length) return;
        events.forEach(event => {
            const node = this.nodes.get(event.entity);
            if (!node) return;

            if (event.type === 'production') {
                node.pulse = 1.0;
                if (this.vfxEnabled) {
                    this.shockwaves.push({
                        x: node.x, y: node.y, r: node.radius, maxR: node.radius + 38,
                        color: '#46d29a', alpha: 0.9, width: 2.0,
                    });
                    this.floaties.push({
                        text: `+${compactNumber(event.amount)} ${prettyName(event.resource)}`,
                        x: node.x, y: node.y - node.radius - 8,
                        vy: -1.2, color: '#46d29a', alpha: 1, life: 60, maxLife: 60,
                    });
                }
            } else if (event.type === 'shortage') {
                node.warningPulse = 1.0;
                if (this.vfxEnabled) {
                    this.shockwaves.push({
                        x: node.x, y: node.y, r: node.radius, maxR: node.radius + 50,
                        color: '#ff6f7c', alpha: 1.0, width: 2.6,
                    });
                    this.floaties.push({
                        text: `⚠ SHORTAGE: ${prettyName(event.resource)}`,
                        x: node.x, y: node.y - node.radius - 12,
                        vy: -0.9, color: '#ff6f7c', alpha: 1, life: 80, maxLife: 80,
                    });
                }
            } else if (event.type === 'capacity_change') {
                if (this.vfxEnabled) {
                    this.shockwaves.push({
                        x: node.x, y: node.y, r: node.radius, maxR: node.radius + 32,
                        color: '#f1b96b', alpha: 0.85, width: 2.0,
                    });
                    this.floaties.push({
                        text: `⚡ ${Math.round(event.value * 100)}%`,
                        x: node.x, y: node.y - node.radius - 8,
                        vy: -1.0, color: '#f1b96b', alpha: 1, life: 65, maxLife: 65,
                    });
                }
            }
        });
    },

    setSelectedEntity(name) {
        this.selectedNodeName = name;
    },

    getNodeAt(px, py) {
        if (!this.canvas) return null;
        const rect = this.canvas.getBoundingClientRect();
        const cx = rect.width / 2;
        const cy = rect.height / 2;

        const wx = (px - cx - this.camera.x) / this.camera.zoom;
        const wy = (py - cy - this.camera.y) / this.camera.zoom;

        for (const node of this.nodes.values()) {
            const dist = Math.hypot(wx - node.x, wy - node.y);
            if (dist <= node.radius + 8) return node;
        }
        return null;
    },

    checkHover(px, py) {
        const hit = this.getNodeAt(px, py);
        this.hoveredNode = hit;
        this.canvas.style.cursor = hit ? 'pointer' : (this.camera.isDragging ? 'grabbing' : 'grab');
        this.updateHud(hit, px, py);
    },

    updateHud(node, px, py) {
        if (!this.hud) return;
        if (!node) {
            this.hud.classList.add('hidden');
            return;
        }

        el('cg-hud-title').textContent = prettyName(node.name);
        const typeEl = el('cg-hud-type');
        typeEl.textContent = node.theme.label;
        typeEl.style.color = node.theme.color;
        typeEl.style.borderColor = `${node.theme.color}44`;

        el('cg-hud-region').textContent = `Sector: ${prettyName(node.region)}`;

        // Rates
        const ratesEl = el('cg-hud-rates');
        ratesEl.replaceChildren();
        if (node.production) {
            const inps = Object.entries(node.production.inputs || {}).map(([r, v]) => `-${v} ${prettyName(r)}`).join(' · ');
            const outs = Object.entries(node.production.outputs || {}).map(([r, v]) => `+${v} ${prettyName(r)}`).join(' · ');
            if (inps) {
                const rowIn = document.createElement('div');
                rowIn.textContent = `Consumes: ${inps}`;
                ratesEl.append(rowIn);
            }
            if (outs) {
                const rowOut = document.createElement('div');
                rowOut.style.color = 'var(--green)';
                rowOut.textContent = `Produces: ${outs}`;
                ratesEl.append(rowOut);
            }
        }
        if (node.capacity != null) {
            const capRow = document.createElement('div');
            capRow.textContent = `Capacity: ${Math.round(node.capacity * 100)}%`;
            ratesEl.append(capRow);
        }

        // Inventory
        const invEl = el('cg-hud-inventory');
        invEl.replaceChildren();
        const entries = Object.entries(node.inventory);
        if (entries.length) {
            entries.forEach(([resource, amount]) => {
                const row = document.createElement('div');
                row.className = 'cg-hud-inv-row';
                const name = document.createElement('span');
                name.textContent = prettyName(resource);
                const val = document.createElement('strong');
                val.textContent = compactNumber(amount);
                val.style.color = getResourceNeonColor(resource);
                row.append(name, val);
                invEl.append(row);
            });
        } else {
            const empty = document.createElement('span');
            empty.style.color = 'var(--text-muted)';
            empty.textContent = 'Inventory empty';
            invEl.append(empty);
        }

        // Position HUD near entity or cursor
        const pad = 16;
        const rect = this.canvas.getBoundingClientRect();
        let left = px + pad;
        let top = py + pad;
        if (left + 230 > rect.width) left = px - 230 - pad;
        if (top + 180 > rect.height) top = py - 180 - pad;

        this.hud.style.left = `${Math.max(10, left)}px`;
        this.hud.style.top = `${Math.max(10, top)}px`;
        this.hud.classList.remove('hidden');
    },

    render(now) {
        if (!this.canvas || !this.ctx) return;
        const ctx = this.ctx;
        const dpr = window.devicePixelRatio || 1;
        const w = this.canvas.clientWidth;
        const h = this.canvas.clientHeight;

        if (this.canvas.width !== Math.floor(w * dpr) || this.canvas.height !== Math.floor(h * dpr)) {
            this.canvas.width = Math.floor(w * dpr);
            this.canvas.height = Math.floor(h * dpr);
        }

        ctx.save();
        ctx.scale(dpr, dpr);
        ctx.clearRect(0, 0, w, h);

        // Smooth camera lerp
        this.camera.x += (this.camera.targetX - this.camera.x) * 0.16;
        this.camera.y += (this.camera.targetY - this.camera.y) * 0.16;
        this.camera.zoom += (this.camera.targetZoom - this.camera.zoom) * 0.16;

        // Smooth node position lerp
        this.nodes.forEach(node => {
            node.x += (node.targetX - node.x) * 0.12;
            node.y += (node.targetY - node.y) * 0.12;
            if (node.pulse > 0) node.pulse = Math.max(0, node.pulse - 0.025);
            if (node.warningPulse > 0) node.warningPulse = Math.max(0, node.warningPulse - 0.02);
        });

        // 1. Draw ambient background & tactical grid
        this.drawBackground(w, h, now);

        // Transform into world space
        ctx.save();
        ctx.translate(w / 2 + this.camera.x, h / 2 + this.camera.y);
        ctx.scale(this.camera.zoom, this.camera.zoom);

        // 2. Draw conduits
        this.drawConduits(ctx, now);

        // 3. Draw resource packet particles
        if (this.vfxEnabled) {
            this.drawParticles(ctx, now);
        }

        // 4. Draw shockwave ripple effects
        if (this.vfxEnabled) {
            this.drawShockwaves(ctx);
        }

        // 5. Draw entity nodes
        this.drawNodes(ctx, now);

        // 6. Draw floating combat-style text ("floaties")
        if (this.vfxEnabled) {
            this.drawFloaties(ctx);
        }

        ctx.restore();
        ctx.restore();
    },

    drawBackground(w, h, now) {
        const ctx = this.ctx;
        // Central subtle radar sweep line
        this.radarAngle = (now * 0.0006) % (Math.PI * 2);
        const cx = w / 2 + this.camera.x * 0.2;
        const cy = h / 2 + this.camera.y * 0.2;
        const sweepLen = Math.max(w, h) * 0.7;

        ctx.save();
        const sweepGrad = ctx.createLinearGradient(cx, cy, cx + Math.cos(this.radarAngle) * sweepLen, cy + Math.sin(this.radarAngle) * sweepLen);
        sweepGrad.addColorStop(0, 'rgba(66,211,234,0.06)');
        sweepGrad.addColorStop(1, 'transparent');
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.arc(cx, cy, sweepLen, this.radarAngle - 0.35, this.radarAngle);
        ctx.closePath();
        ctx.fillStyle = sweepGrad;
        ctx.fill();

        // Subtle ambient floating dust particles
        if (this.vfxEnabled) {
            this.ambientDust.forEach(dust => {
                dust.x += dust.speedX;
                dust.y += dust.speedY;
                if (dust.x > w / 2 + 300) dust.x = -w / 2 - 300;
                if (dust.x < -w / 2 - 300) dust.x = w / 2 + 300;
                if (dust.y > h / 2 + 200) dust.y = -h / 2 - 200;
                if (dust.y < -h / 2 - 200) dust.y = h / 2 + 200;

                const screenX = w / 2 + dust.x + this.camera.x * 0.4;
                const screenY = h / 2 + dust.y + this.camera.y * 0.4;
                ctx.beginPath();
                ctx.arc(screenX, screenY, dust.r, 0, Math.PI * 2);
                ctx.fillStyle = `${dust.color}${Math.floor(dust.alpha * 255).toString(16).padStart(2, '0')}`;
                ctx.fill();
            });
        }
        ctx.restore();
    },

    getConduitControlPoint(from, to) {
        const dx = to.x - from.x;
        const dy = to.y - from.y;
        const dist = Math.hypot(dx, dy) || 1;
        const nx = -dy / dist;
        const ny = dx / dist;
        const curvature = Math.min(35, Math.max(12, dist * 0.12));
        return {
            x: (from.x + to.x) / 2 + nx * curvature,
            y: (from.y + to.y) / 2 + ny * curvature,
        };
    },

    drawConduits(ctx, now) {
        const flowOffset = (now * 0.04) % 20;
        this.links.forEach(link => {
            const from = this.nodes.get(link.from);
            const to = this.nodes.get(link.to);
            if (!from || !to) return;

            const cp = this.getConduitControlPoint(from, to);

            // Conduit dark base shield
            ctx.beginPath();
            ctx.moveTo(from.x, from.y);
            ctx.quadraticCurveTo(cp.x, cp.y, to.x, to.y);
            ctx.strokeStyle = 'rgba(7,14,24,0.7)';
            ctx.lineWidth = 4.5;
            ctx.stroke();

            // Conduit neon core
            ctx.beginPath();
            ctx.moveTo(from.x, from.y);
            ctx.quadraticCurveTo(cp.x, cp.y, to.x, to.y);
            ctx.strokeStyle = `${link.color}40`;
            ctx.lineWidth = 1.8;
            ctx.stroke();

            // Flow pulses
            ctx.save();
            ctx.setLineDash([4, 12]);
            ctx.lineDashOffset = -flowOffset;
            ctx.strokeStyle = `${link.color}aa`;
            ctx.lineWidth = 1.4;
            ctx.stroke();
            ctx.restore();
        });
    },

    drawParticles(ctx, now) {
        this.particles.forEach(p => {
            const from = this.nodes.get(p.link.from);
            const to = this.nodes.get(p.link.to);
            if (!from || !to) return;

            p.t += p.speed;
            if (p.t >= 1) {
                p.t = 0;
                // Micro absorption spark at destination
                if (Math.random() < 0.25) {
                    this.shockwaves.push({
                        x: to.x, y: to.y, r: to.radius - 4, maxR: to.radius + 12,
                        color: p.color, alpha: 0.5, width: 1.2,
                    });
                }
            }

            const cp = this.getConduitControlPoint(from, to);
            const t = p.t;
            const px = (1 - t) * (1 - t) * from.x + 2 * (1 - t) * t * cp.x + t * t * to.x;
            const py = (1 - t) * (1 - t) * from.y + 2 * (1 - t) * t * cp.y + t * t * to.y;

            // Tail
            const t0 = Math.max(0, t - 0.05);
            const tx0 = (1 - t0) * (1 - t0) * from.x + 2 * (1 - t0) * t0 * cp.x + t0 * t0 * to.x;
            const ty0 = (1 - t0) * (1 - t0) * from.y + 2 * (1 - t0) * t0 * cp.y + t0 * t0 * to.y;

            ctx.beginPath();
            ctx.moveTo(tx0, ty0);
            ctx.lineTo(px, py);
            ctx.strokeStyle = `${p.color}55`;
            ctx.lineWidth = p.r * 1.5;
            ctx.stroke();

            // Head
            ctx.beginPath();
            ctx.arc(px, py, p.r, 0, Math.PI * 2);
            ctx.fillStyle = '#ffffff';
            ctx.shadowColor = p.color;
            ctx.shadowBlur = 8;
            ctx.fill();
            ctx.shadowBlur = 0;
        });
    },

    drawShockwaves(ctx) {
        for (let i = this.shockwaves.length - 1; i >= 0; i--) {
            const sw = this.shockwaves[i];
            sw.r += (sw.maxR - sw.r) * 0.14 + 0.8;
            sw.alpha *= 0.91;

            if (sw.alpha < 0.05 || sw.r >= sw.maxR) {
                this.shockwaves.splice(i, 1);
                continue;
            }

            ctx.save();
            ctx.beginPath();
            ctx.arc(sw.x, sw.y, sw.r, 0, Math.PI * 2);
            ctx.strokeStyle = `${sw.color}${Math.floor(sw.alpha * 255).toString(16).padStart(2, '0')}`;
            ctx.lineWidth = sw.width;
            ctx.shadowColor = sw.color;
            ctx.shadowBlur = 10;
            ctx.stroke();
            ctx.restore();
        }
    },

    drawFloaties(ctx) {
        ctx.font = '600 11px ui-monospace, SFMono-Regular, monospace';
        ctx.textAlign = 'center';
        for (let i = this.floaties.length - 1; i >= 0; i--) {
            const fl = this.floaties[i];
            fl.y += fl.vy;
            fl.vy *= 0.97;
            fl.life--;
            fl.alpha = Math.max(0, fl.life / fl.maxLife);

            if (fl.life <= 0) {
                this.floaties.splice(i, 1);
                continue;
            }

            ctx.save();
            ctx.fillStyle = `${fl.color}${Math.floor(fl.alpha * 255).toString(16).padStart(2, '0')}`;
            ctx.shadowColor = '#000000';
            ctx.shadowBlur = 5;
            ctx.fillText(fl.text, fl.x, fl.y);
            ctx.restore();
        }
    },

    drawNodes(ctx, now) {
        this.nodes.forEach(node => {
            const isHovered = this.hoveredNode === node;
            const isSelected = this.selectedNodeName === node.name;
            const r = node.radius;

            ctx.save();
            ctx.translate(node.x, node.y);

            // 1. Node base glow
            const glowR = r + (node.pulse * 14) + (isHovered ? 8 : 4);
            const baseGlow = ctx.createRadialGradient(0, 0, r * 0.4, 0, 0, glowR);
            baseGlow.addColorStop(0, `${node.theme.color}35`);
            baseGlow.addColorStop(1, `${node.theme.color}00`);
            ctx.beginPath();
            ctx.arc(0, 0, glowR, 0, Math.PI * 2);
            ctx.fillStyle = baseGlow;
            ctx.fill();

            // 2. Rotating tactical corner brackets
            const bracketRot = (now * 0.0008) % (Math.PI * 2);
            ctx.save();
            ctx.rotate(bracketRot);
            ctx.strokeStyle = isSelected ? '#f1b96b' : `${node.theme.color}77`;
            ctx.lineWidth = 1.4;
            const bR = r + 4;
            for (let b = 0; b < 4; b++) {
                ctx.beginPath();
                ctx.arc(0, 0, bR, b * Math.PI / 2 + 0.12, (b + 1) * Math.PI / 2 - 0.12);
                ctx.stroke();
            }
            ctx.restore();

            // 3. Base plate
            ctx.beginPath();
            ctx.arc(0, 0, r, 0, Math.PI * 2);
            ctx.fillStyle = '#0a101b';
            ctx.fill();
            ctx.strokeStyle = isSelected ? '#f1b96b' : (node.warningPulse > 0.1 ? '#ff6f7c' : `${node.theme.color}bb`);
            ctx.lineWidth = isSelected ? 2.6 : 2;
            ctx.shadowColor = isSelected ? '#f1b96b' : node.theme.color;
            ctx.shadowBlur = isSelected ? 12 : 6;
            ctx.stroke();
            ctx.shadowBlur = 0;

            // 4. Radial inventory ring gauge
            const invEntries = Object.entries(node.inventory);
            const totalInv = invEntries.reduce((sum, [, val]) => sum + val, 0);
            if (totalInv > 0) {
                let startAngle = -Math.PI / 2;
                invEntries.forEach(([res, val]) => {
                    const slice = (val / totalInv) * Math.PI * 2;
                    ctx.beginPath();
                    ctx.arc(0, 0, r - 3, startAngle, startAngle + slice);
                    ctx.strokeStyle = getResourceNeonColor(res);
                    ctx.lineWidth = 2.4;
                    ctx.stroke();
                    startAngle += slice;
                });
            }

            // 5. Archetype procedural CG graphic icon
            this.drawArchetypeGraphic(ctx, node.archetype, node.theme.color, now, node.capacity);

            // 6. Overdrive sparks
            if (node.capacity > 1.05 && this.vfxEnabled) {
                this.drawOverdriveSparks(ctx, r, now);
            }

            // 7. Hazard warning overlay
            if (node.warningPulse > 0.05) {
                ctx.beginPath();
                ctx.arc(0, 0, r + 8, 0, Math.PI * 2);
                ctx.strokeStyle = `rgba(255,111,124,${node.warningPulse})`;
                ctx.lineWidth = 2;
                ctx.stroke();
            }

            // 8. Capacity badge pill
            if (node.capacity != null) {
                const capPct = Math.round(node.capacity * 100);
                const capText = `${capPct}%`;
                ctx.font = '700 8px ui-monospace, monospace';
                const tw = ctx.measureText(capText).width;
                ctx.fillStyle = node.capacity > 1.0 ? 'rgba(241,185,107,0.9)' : (node.capacity === 0 ? 'rgba(255,111,124,0.85)' : 'rgba(8,12,20,0.85)');
                ctx.strokeStyle = node.capacity > 1.0 ? '#f1b96b' : (node.capacity === 0 ? '#ff6f7c' : 'rgba(103,169,255,0.4)');
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.roundRect(-tw / 2 - 4, -r - 11, tw + 8, 12, 3);
                ctx.fill();
                ctx.stroke();
                ctx.fillStyle = node.capacity > 1.0 ? '#000000' : '#dce5f2';
                ctx.textAlign = 'center';
                ctx.fillText(capText, 0, -r - 2);
            }

            // 9. Label and Inventory Count
            ctx.font = '600 11px ui-sans-serif, system-ui, sans-serif';
            ctx.textAlign = 'center';
            const nameText = prettyName(node.name);
            const nameWidth = ctx.measureText(nameText).width;

            // Name plate background
            ctx.fillStyle = 'rgba(7,11,18,0.82)';
            ctx.beginPath();
            ctx.roundRect(-nameWidth / 2 - 5, r + 7, nameWidth + 10, 16, 4);
            ctx.fill();
            ctx.strokeStyle = isSelected ? '#f1b96b' : 'rgba(163,184,214,0.18)';
            ctx.lineWidth = 1;
            ctx.stroke();

            ctx.fillStyle = isSelected ? '#f1b96b' : '#e2e8f0';
            ctx.fillText(nameText, 0, r + 19);

            // Subtitle inventory
            ctx.font = '500 9px ui-monospace, monospace';
            ctx.fillStyle = '#74839a';
            ctx.fillText(`${compactNumber(totalInv)} units`, 0, r + 33);

            // 10. Selected Reticle
            if (isSelected) {
                this.drawReticle(ctx, r + 10, now);
            }

            ctx.restore();
        });
    },

    drawArchetypeGraphic(ctx, archetype, color, now, capacity) {
        ctx.save();
        ctx.fillStyle = color;
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.6;

        switch (archetype) {
            case 'power': {
                // Spinning 3-blade turbine
                const angle = now * 0.003 * (capacity || 1);
                ctx.beginPath();
                ctx.arc(0, 0, 3.5, 0, Math.PI * 2);
                ctx.fill();
                for (let i = 0; i < 3; i++) {
                    const a = angle + i * (Math.PI * 2 / 3);
                    ctx.beginPath();
                    ctx.moveTo(Math.cos(a) * 3, Math.sin(a) * 3);
                    ctx.lineTo(Math.cos(a + 0.35) * 11, Math.sin(a + 0.35) * 11);
                    ctx.lineTo(Math.cos(a) * 13, Math.sin(a) * 13);
                    ctx.closePath();
                    ctx.fill();
                }
                break;
            }
            case 'extractor': {
                // Reciprocating drill triangle with piston
                const yOff = Math.sin(now * 0.007) * 3;
                ctx.beginPath();
                ctx.moveTo(0, 9 + yOff);
                ctx.lineTo(-7, -4 + yOff);
                ctx.lineTo(7, -4 + yOff);
                ctx.closePath();
                ctx.stroke();
                ctx.fillRect(-2, -9, 4, 6 + yOff);
                break;
            }
            case 'processor': {
                // Crucible vat / foundry with molten flame
                ctx.beginPath();
                ctx.moveTo(-8, -6);
                ctx.lineTo(8, -6);
                ctx.lineTo(5, 7);
                ctx.lineTo(-5, 7);
                ctx.closePath();
                ctx.stroke();
                // Molten core
                ctx.beginPath();
                ctx.arc(0, 1 + Math.sin(now * 0.008) * 1.5, 3, 0, Math.PI * 2);
                ctx.fill();
                break;
            }
            case 'manufacturer': {
                // Dual interlocking cogs
                const a1 = now * 0.0025;
                ctx.save();
                ctx.translate(-3, 0);
                ctx.rotate(a1);
                this.drawCog(ctx, 7, 6);
                ctx.restore();
                ctx.save();
                ctx.translate(6, 4);
                ctx.rotate(-a1 + 0.4);
                this.drawCog(ctx, 5, 5);
                ctx.restore();
                break;
            }
            case 'depot': {
                // Isometric storage cube
                ctx.beginPath();
                ctx.moveTo(0, -9);
                ctx.lineTo(8, -4.5);
                ctx.lineTo(8, 4.5);
                ctx.lineTo(0, 9);
                ctx.lineTo(-8, 4.5);
                ctx.lineTo(-8, -4.5);
                ctx.closePath();
                ctx.stroke();
                ctx.beginPath();
                ctx.moveTo(0, -9); ctx.lineTo(0, 0);
                ctx.moveTo(0, 0); ctx.lineTo(8, -4.5);
                ctx.moveTo(0, 0); ctx.lineTo(-8, -4.5);
                ctx.moveTo(0, 0); ctx.lineTo(0, 9);
                ctx.stroke();
                break;
            }
            case 'habitat': {
                // City towers silhouette
                ctx.fillRect(-8, -3, 4, 11);
                ctx.fillRect(-3, -9, 6, 17);
                ctx.fillRect(4, -5, 4, 13);
                ctx.fillStyle = '#ffffff';
                ctx.fillRect(-1, -7, 2, 2);
                ctx.fillRect(-1, -3, 2, 2);
                ctx.fillRect(5, -3, 2, 2);
                break;
            }
            case 'bio': {
                // Organic double leaf / helix
                ctx.beginPath();
                ctx.ellipse(0, 0, 4, 9, Math.PI / 4, 0, Math.PI * 2);
                ctx.stroke();
                ctx.beginPath();
                ctx.ellipse(0, 0, 4, 9, -Math.PI / 4, 0, Math.PI * 2);
                ctx.stroke();
                ctx.beginPath();
                ctx.arc(0, 0, 2.5, 0, Math.PI * 2);
                ctx.fill();
                break;
            }
            case 'water': {
                // Undulating waves
                const wOff = now * 0.005;
                for (let row = -3; row <= 5; row += 4) {
                    ctx.beginPath();
                    for (let x = -8; x <= 8; x += 2) {
                        const y = row + Math.sin(x * 0.5 + wOff) * 2;
                        if (x === -8) ctx.moveTo(x, y);
                        else ctx.lineTo(x, y);
                    }
                    ctx.stroke();
                }
                break;
            }
            case 'care': {
                // Medical cross
                ctx.fillRect(-3, -8, 6, 16);
                ctx.fillRect(-8, -3, 16, 6);
                break;
            }
            default: {
                // Tech diamond
                ctx.beginPath();
                ctx.moveTo(0, -8);
                ctx.lineTo(8, 0);
                ctx.lineTo(0, 8);
                ctx.lineTo(-8, 0);
                ctx.closePath();
                ctx.stroke();
                ctx.beginPath();
                ctx.arc(0, 0, 3, 0, Math.PI * 2);
                ctx.fill();
                break;
            }
        }
        ctx.restore();
    },

    drawCog(ctx, r, teeth) {
        ctx.beginPath();
        for (let i = 0; i < teeth * 2; i++) {
            const angle = (i * Math.PI) / teeth;
            const dist = i % 2 === 0 ? r : r - 2.5;
            const x = Math.cos(angle) * dist;
            const y = Math.sin(angle) * dist;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        }
        ctx.closePath();
        ctx.stroke();
    },

    drawOverdriveSparks(ctx, r, now) {
        ctx.save();
        ctx.strokeStyle = '#42d3ea';
        ctx.lineWidth = 1.5;
        const sparkCount = 3;
        for (let i = 0; i < sparkCount; i++) {
            const a = (now * 0.01 + i * 2.1) % (Math.PI * 2);
            const dist = r + 3 + (Math.sin(now * 0.02 + i) * 3);
            const x = Math.cos(a) * dist;
            const y = Math.sin(a) * dist;
            ctx.beginPath();
            ctx.moveTo(x, y);
            ctx.lineTo(x + (Math.random() - 0.5) * 8, y + (Math.random() - 0.5) * 8);
            ctx.stroke();
        }
        ctx.restore();
    },

    drawReticle(ctx, r, now) {
        ctx.save();
        ctx.strokeStyle = '#f1b96b';
        ctx.lineWidth = 1.8;
        const cornerSize = 7;
        // 4 corner targeting brackets
        [
            [-r, -r, 1, 1],
            [r, -r, -1, 1],
            [-r, r, 1, -1],
            [r, r, -1, -1],
        ].forEach(([x, y, dx, dy]) => {
            ctx.beginPath();
            ctx.moveTo(x + dx * cornerSize, y);
            ctx.lineTo(x, y);
            ctx.lineTo(x, y + dy * cornerSize);
            ctx.stroke();
        });

        // Pulsing outer tick
        ctx.setLineDash([2, 8]);
        ctx.lineDashOffset = -(now * 0.02);
        ctx.beginPath();
        ctx.arc(0, 0, r + 5, 0, Math.PI * 2);
        ctx.stroke();
        ctx.restore();
    },
};

initialize();
