/** World Forge Simulation Studio — dependency-free dashboard client. */

const COLORS = ['#67a9ff', '#42d3ea', '#9b8cff', '#46d29a', '#f1b96b', '#ff6f7c', '#d578ef'];
const TYPE_COLORS = {
    production: '#46d29a',
    transfer: '#67a9ff',
    shortage: '#ff6f7c',
    price: '#f1b96b',
    system: '#9b8cff',
    construction: '#f1b96b',
    research: '#d578ef',
    governance: '#42d3ea',
    geopolitics: '#ff6f7c',
    intrigue: '#f1b96b',
};
const WORLD_SYMBOLS = {
    'supply-chain': 'SC', ecosystem: 'EC', 'micro-city': 'MC',
    'freight-network': 'FN', 'stress-test': 'ST', 'minimal-world': 'MW',
    'coastal-resilience': 'CR', 'supply-chain-recovery': 'SR',
};

const THEME_LABELS = {
    'cyber-tactical': 'Cyber Tactical',
    'solaris-gold': 'Solaris Gold',
    'bio-synthetic': 'Bio Synthetic',
    'cryo-vector': 'Cryo Vector',
};

const FONT_SANS = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";
const FONT_MONO = "'JetBrains Mono', ui-monospace, 'SFMono-Regular', Consolas, monospace";

function getWorldFlagSvg(worldId, size = 20) {
    const s = size;
    const flags = {
        'supply-chain': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#0b1322"/><path d="M4 4 L20 4 L20 16 L12 21 L4 16 Z" fill="none" stroke="#67a9ff" stroke-width="1.2"/><circle cx="12" cy="11" r="4.5" fill="none" stroke="#46d29a" stroke-width="1.4" stroke-dasharray="3 1.5"/><circle cx="12" cy="11" r="1.5" fill="#42d3ea"/><path d="M8.5 16.5 L12 19 L15.5 16.5" fill="none" stroke="#f1b96b" stroke-width="1.5" stroke-linecap="round"/></svg>`,
        'supply-chain-recovery': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#09150f"/><path d="M5 7h9l-2 3 2 3H5z" fill="none" stroke="#46d29a" stroke-width="1.3"/><path d="M5 4v16M8 18h10" fill="none" stroke="#67a9ff" stroke-width="1.3" stroke-linecap="round"/><path d="m15 14 2 2 3-4" fill="none" stroke="#f1b96b" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
        'coastal-resilience': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#04121d"/><path d="M12 3 L19 7 L19 14 C19 18 12 21 12 21 C12 21 5 18 5 14 L5 7 Z" fill="none" stroke="#0ea5e9" stroke-width="1.2"/><path d="M7 13 Q9.5 10 12 12.5 Q14.5 15 17 12" fill="none" stroke="#38bdf8" stroke-width="1.6" stroke-linecap="round"/><circle cx="12" cy="7.5" r="1.6" fill="#e0f2fe"/></svg>`,
        'ecosystem': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#04160c"/><circle cx="12" cy="12" r="8" fill="none" stroke="#10b981" stroke-width="1.2"/><path d="M12 17 C12 12.5 8 10.5 8 7.5 C11 7.5 12 10.5 12 10.5 C12 10.5 13 7.5 16 7.5 C16 10.5 12 12.5 12 17 Z" fill="#46d29a"/><circle cx="12" cy="5.5" r="1.5" fill="#facc15"/></svg>`,
        'micro-city': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#140c06"/><path d="M5 19 L5 12 L9 9 L15 9 L19 13 L19 19 Z" fill="none" stroke="#f59e0b" stroke-width="1.2"/><rect x="6.5" y="11" width="3" height="8" fill="#fbbf24" fill-opacity="0.6"/><rect x="10.5" y="6" width="3" height="13" fill="#fbbf24"/><rect x="14.5" y="10" width="3" height="9" fill="#fbbf24" fill-opacity="0.6"/><circle cx="12" cy="4.5" r="1.2" fill="#fef08a"/></svg>`,
        'freight-network': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#081120"/><circle cx="12" cy="12" r="8" fill="none" stroke="#3b82f6" stroke-width="1.2"/><polygon points="12,5 14,10 19,12 14,14 12,19 10,14 5,12 10,10" fill="#f97316"/><circle cx="12" cy="12" r="2" fill="#ffffff"/></svg>`,
        'stress-test': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#1c0707"/><polygon points="12,3 21,8.5 21,15.5 12,21 3,15.5 3,8.5" fill="none" stroke="#ef4444" stroke-width="1.2"/><path d="M13 5.5 L8 12.5 L12 12.5 L11 18.5 L16 11.5 L12 11.5 Z" fill="#fbbf24"/></svg>`,
        'minimal-world': `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#0f0a1c"/><polygon points="12,4 19,18 5,18" fill="none" stroke="#c084fc" stroke-width="1.3"/><circle cx="12" cy="13" r="2.5" fill="#38bdf8"/></svg>`,
    };
    return flags[worldId] || `<svg viewBox="0 0 24 24" width="${s}" height="${s}" class="world-flag-svg" aria-hidden="true"><rect width="24" height="24" rx="4" fill="#090e17"/><circle cx="12" cy="12" r="5" fill="none" stroke="#38bdf8" stroke-width="1.3"/><ellipse cx="12" cy="12" rx="9" ry="3.5" transform="rotate(-28 12 12)" fill="none" stroke="#67a9ff" stroke-width="1.1" stroke-dasharray="2 2"/><circle cx="12" cy="12" r="2" fill="#42d3ea"/></svg>`;
}

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
    analytics: { report: null, activeResource: null },
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
    analyticsDialog: el('analytics-dialog'),
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
    el('nav-analytics')?.addEventListener('click', openAnalytics);
    el('nav-benchmark').addEventListener('click', openBenchmark);
    el('nav-play').addEventListener('click', openPlayMode);
    el('nav-builder').addEventListener('click', openWorldBuilder);
    el('compare-run').addEventListener('click', runComparison);
    el('analytics-run-btn')?.addEventListener('click', runAnalytics);
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
    el('city-building-select')?.addEventListener('change', syncCityBuildingSelection);
    el('city-district-select')?.addEventListener('change', syncCityBuildingSelection);
    el('city-build-button')?.addEventListener('click', constructCityBuilding);
    document.querySelectorAll('.preset-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const val = Number(btn.dataset.capacity);
            el('capacity-slider').value = val;
            updateCapacityLabel();
            syncPresetHighlight(val);
            if (val > 100) WorldForgeCG.audio.playOverdrive();
            else WorldForgeCG.audio.playBlip(600, 0.05);
        });
    });
    initThemeManager();
    WorldForgeCG.bindControls();
    el('save-close').addEventListener('click', () => dom.saveDialog.close());
    el('save-form').addEventListener('submit', saveCurrentSession);
    el('refresh-saves').addEventListener('click', refreshSaves);
    el('builder-close').addEventListener('click', () => dom.builderDialog.close());
    el('analytics-close')?.addEventListener('click', () => dom.analyticsDialog.close());
    el('builder-form').addEventListener('submit', createWorld);
    el('builder-world-title').addEventListener('input', syncBuilderSlug);
    el('builder-world-id').addEventListener('input', () => { state.builderIdTouched = true; });
    el('builder-description').addEventListener('input', updateBuilderDescriptionCount);
    el('menu-button').addEventListener('click', () => toggleSidebar(true));
    el('sidebar-close').addEventListener('click', () => toggleSidebar(false));
    el('mobile-backdrop').addEventListener('click', () => toggleSidebar(false));
    el('val-fingerprint').addEventListener('click', () => copyProof('final'));
    el('nav-trees')?.addEventListener('click', openTreeModal);
    el('btn-fullscreen-trees')?.addEventListener('click', openTreeModal);
    el('tree-modal-close')?.addEventListener('click', closeTreeModal);
    el('nav-trophies')?.addEventListener('click', openTrophiesModal);
    el('btn-achievements-dock')?.addEventListener('click', openTrophiesModal);
    el('trophies-close')?.addEventListener('click', closeTrophiesModal);
    el('level-up-dismiss')?.addEventListener('click', closeLevelUpBanner);
    el('btn-header-play-game')?.addEventListener('click', launchPlayGameMode);
    el('btn-hud-posture')?.addEventListener('click', toggleDefensePosture);
    el('btn-war-room-posture')?.addEventListener('click', toggleDefensePosture);
    el('btn-hud-war-room')?.addEventListener('click', openWarRoomModal);
    el('war-room-close')?.addEventListener('click', closeWarRoomModal);
    setupBuildDock();
    setupTreeTabs();
    setupCommanderPowers();
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

function syncPresetHighlight(val) {
    const num = Number(val);
    document.querySelectorAll('.preset-btn').forEach(btn => {
        btn.classList.toggle('active', Number(btn.dataset.capacity) === num);
    });
}

function initThemeManager() {
    const selector = el('theme-selector');
    const menuBtn = el('theme-menu-btn');
    const dropdown = el('theme-dropdown');
    const label = el('current-theme-label');
    if (!selector || !menuBtn || !dropdown) return;

    function setTheme(theme) {
        const validTheme = THEME_LABELS[theme] ? theme : 'cyber-tactical';
        document.documentElement.setAttribute('data-theme', validTheme);
        try { localStorage.setItem('worldforge.theme', validTheme); } catch (_) {}
        if (label) label.textContent = THEME_LABELS[validTheme];
        document.querySelectorAll('.theme-option').forEach(opt => {
            opt.classList.toggle('active', opt.dataset.theme === validTheme);
        });
    }

    menuBtn.addEventListener('click', e => {
        e.stopPropagation();
        const isOpen = !dropdown.classList.contains('hidden');
        dropdown.classList.toggle('hidden', isOpen);
        menuBtn.setAttribute('aria-expanded', String(!isOpen));
    });

    document.querySelectorAll('.theme-option').forEach(opt => {
        opt.addEventListener('click', () => {
            setTheme(opt.dataset.theme);
            dropdown.classList.add('hidden');
            menuBtn.setAttribute('aria-expanded', 'false');
        });
    });

    window.addEventListener('click', e => {
        if (!selector.contains(e.target)) {
            dropdown.classList.add('hidden');
            menuBtn.setAttribute('aria-expanded', 'false');
        }
    });

    let savedTheme = 'cyber-tactical';
    try { savedTheme = localStorage.getItem('worldforge.theme') || 'cyber-tactical'; } catch (_) {}
    setTheme(savedTheme);
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
        icon.innerHTML = getWorldFlagSvg(world.id, 20);
        const copy = document.createElement('span');
        copy.className = 'nav-copy';
        const title = document.createElement('strong');
        title.textContent = world.title;
        const details = document.createElement('small');
        const baseCount = world.dependencies?.length || 0;
        details.textContent = `${world.entityCount} entities · ${world.resourceCount} resources${world.technologyCount ? ` · ${world.technologyCount} techs` : ''}${baseCount ? ` · ${baseCount} base` : ''}`;
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
    const baseCount = world.dependencies?.length || 0;
    dom.worldMeta.textContent = `v${world.version} · ${world.entityCount} entities · ${world.resourceCount} resources${world.cityBuildingCount ? ` · ${world.cityBuildingCount} buildings · ${world.technologyCount} techs` : ''}${baseCount ? ` · ${baseCount} inherited base` : ''}`;
    const flagBadge = el('world-flag-badge');
    if (flagBadge) flagBadge.innerHTML = getWorldFlagSvg(worldId, 18);
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
    updateVitalityGauge(data);
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
    updateVitalityGauge(null);
}

function updateVitalityGauge(data) {
    const circle = el('vitality-circle');
    const scoreEl = el('vitality-score');
    const statusEl = el('vitality-status');
    if (!circle || !scoreEl || !statusEl) return;

    if (!data) {
        circle.setAttribute('stroke-dasharray', '100, 100');
        circle.className = 'vitality-val';
        scoreEl.textContent = '100%';
        statusEl.textContent = 'Optimal';
        return;
    }

    let score = 100;
    const objectives = data.objectives || [];
    if (objectives.length > 0) {
        let totalProg = 0;
        let failedCount = 0;
        objectives.forEach(obj => {
            const st = (obj.status || '').toLowerCase();
            if (st === 'passed') totalProg += 1.0;
            else if (st === 'failed') failedCount++;
            else totalProg += Math.max(0, Math.min(1, Number(obj.progress) || 0));
        });
        const avgProg = totalProg / objectives.length;
        score = Math.round(avgProg * 100) - (failedCount * 20);
    }

    // Shortage penalty
    const events = data.events || state.play.events || [];
    const recentShortages = events.slice(-25).filter(e => e.type === 'shortage').length;
    score -= recentShortages * 5;

    score = Math.max(5, Math.min(100, score));

    circle.setAttribute('stroke-dasharray', `${score}, 100`);
    circle.className = 'vitality-val';
    if (score < 50) {
        circle.classList.add('critical');
        statusEl.textContent = 'Critical';
    } else if (score < 80) {
        circle.classList.add('warning');
        statusEl.textContent = 'Strained';
    } else {
        statusEl.textContent = 'Optimal';
    }
    scoreEl.textContent = `${score}%`;
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
    el('city-build-button').disabled = true;
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
        renderCityLayer(state.play.data?.city);
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
        renderCityLayer(play.data?.city);
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
    el('city-build-button').disabled = true;
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
        if (capacity > 1.0) WorldForgeCG.audio.playOverdrive();
        else WorldForgeCG.audio.playBlip(680, 0.08);
    } catch (error) {
        showToast('Decision rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        el('apply-capacity').disabled = Boolean(play.data?.completed);
    }
}

async function constructCityBuilding() {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed || !play.data?.city) return;
    const building = el('city-building-select').value;
    const district = el('city-district-select').value;
    if (!building || !district) return;
    play.requestInFlight = true;
    el('city-build-button').disabled = true;
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/construct`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ building, district }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Building constructed', `${prettyName(building)} is now active in ${prettyName(district)}.`);
        WorldForgeCG.audio?.playConstruction?.();
        spawnFloatingText(`🏗️ CONSTRUCTED: ${prettyName(building)}`, null, null, 'fx-surge');
    } catch (error) {
        showToast('Construction rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

async function researchCityTechnology(technology) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed || !play.data?.city) return;
    play.requestInFlight = true;
    renderCityLayer(play.data.city);
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/research`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ technology }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Technology unlocked', `${prettyName(technology)} has reshaped the city system.`);
        WorldForgeCG.audio?.playUnlock?.();
        spawnFloatingText(`🔬 UNLOCKED: ${prettyName(technology)}`, null, null, 'fx-success');
    } catch (error) {
        showToast('Research rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

async function makeCivicDecision(dilemma, option) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed || !play.data?.city) return;
    play.requestInFlight = true;
    renderCityLayer(play.data.city);
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/decide`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ dilemma, option }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Civic mandate adopted', `${prettyName(option)} now shapes the city.`);
        WorldForgeCG.audio?.playLevelUp?.();
        spawnFloatingText(`⚖️ CIVIC MANDATE: ${prettyName(option)}`, null, null, 'fx-culture');
    } catch (error) {
        showToast('Council decision rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

async function executeGeopoliticalAction(action, entity, option = null) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed) return;
    play.requestInFlight = true;
    renderCityLayer(play.data.city);
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/geopolitics`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ action, entity, option }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Strategic order executed', `${prettyName(action)} changed the balance of power.`);
        if (action === 'tribute') WorldForgeCG.audio?.playTribute?.();
        else if (action === 'strike') WorldForgeCG.audio?.playBattleClash?.();
        else if (action === 'posture') WorldForgeCG.audio?.playServoLock?.();
        else if (action === 'emissary' || action === 'coalition') WorldForgeCG.audio?.playWarHorn?.();
        else WorldForgeCG.audio?.playOverdrive?.();
    } catch (error) {
        showToast('Strategic order rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

async function constructCityBuildingDirect(building, district) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed || !play.data?.city) return;
    play.requestInFlight = true;
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/construct`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ building, district }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Building constructed', `${prettyName(building)} active in ${prettyName(district)}.`);
        WorldForgeCG.audio?.playConstruction?.();
        spawnFloatingText(`🏗️ CONSTRUCTED: ${prettyName(building)}`, null, null, 'fx-surge');
    } catch (error) {
        showToast('Construction rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

async function upgradeCityBuildingDirect(building) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed || !play.data?.city) return;
    play.requestInFlight = true;
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/upgrade`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ building }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Building Upgraded', `${prettyName(building)} elevated to higher output tier.`);
        WorldForgeCG.audio?.playConstruction?.();
        spawnFloatingText(`⭐ UPGRADED: ${prettyName(building)}`, null, null, 'fx-surge');
        // Refresh open inspector if open
        const dialog = el('building-inspector-dialog');
        if (dialog && dialog.open && dialog.dataset.building === building) {
            openBuildingInspector(building, dialog.dataset.district);
        }
    } catch (error) {
        showToast('Upgrade rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

async function demolishCityBuildingDirect(building, district) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed || !play.data?.city) return;
    play.requestInFlight = true;
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/demolish`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ building, district }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        showToast('Structure Demolished', `${prettyName(building)} recycled in ${prettyName(district)} (40% cost refunded).`);
        WorldForgeCG.audio?.playShortage?.();
        spawnFloatingText(`♻️ DEMOLISHED: ${prettyName(building)}`, null, null, 'fx-shortage');
        closeBuildingInspector();
    } catch (error) {
        showToast('Demolition rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

function openBuildingInspector(buildingId, districtId) {
    const dialog = el('building-inspector-dialog');
    if (!dialog) return;
    const city = state.play?.data?.city;
    if (!city) return;
    const bDef = city.buildings?.find(b => b.id === buildingId);
    if (!bDef) return;

    dialog.dataset.building = buildingId;
    dialog.dataset.district = districtId || (bDef.allowed_districts?.[0] || 'civic-core');

    const level = bDef.level || 1;
    const stars = level === 1 ? '★☆☆' : (level === 2 ? '★★☆' : '★★★');
    const bonusPct = (level - 1) * 50;

    const badge = el('inspector-badge');
    if (badge) badge.textContent = (bDef.category || 'RESIDENTIAL').toUpperCase();
    const title = el('inspector-title');
    if (title) title.textContent = bDef.name;
    const starsEl = el('inspector-stars');
    if (starsEl) starsEl.textContent = stars;
    const distEl = el('inspector-district');
    if (distEl) distEl.textContent = prettyName(dialog.dataset.district);
    const lvlEl = el('inspector-level');
    if (lvlEl) lvlEl.textContent = `Level ${level} / 3`;
    const multEl = el('inspector-multiplier');
    if (multEl) multEl.textContent = `+${bonusPct}% Output & Capacity`;
    const cntEl = el('inspector-count');
    if (cntEl) cntEl.textContent = `${bDef.count} built (${bDef.maxCount} max)`;
    const descEl = el('inspector-desc');
    if (descEl) descEl.textContent = bDef.description || '';

    // Outputs
    const outputsEl = el('inspector-outputs');
    if (outputsEl) {
        outputsEl.replaceChildren();
        const entries = Object.entries(bDef.outputs || {});
        if (entries.length) {
            entries.forEach(([r, v]) => {
                const item = document.createElement('span');
                item.className = 'stat-item positive';
                const scaled = (v * (1 + bonusPct / 100)).toFixed(1);
                item.textContent = `+${scaled} ${prettyName(r)} / tick`;
                outputsEl.append(item);
            });
        } else {
            outputsEl.innerHTML = '<span class="stat-item">No direct outputs</span>';
        }
    }

    // Upkeep
    const upkeepEl = el('inspector-upkeep');
    if (upkeepEl) {
        upkeepEl.replaceChildren();
        const entries = Object.entries(bDef.upkeep || {});
        if (entries.length) {
            entries.forEach(([r, v]) => {
                const item = document.createElement('span');
                item.className = 'stat-item negative';
                item.textContent = `-${v} ${prettyName(r)} / tick`;
                upkeepEl.append(item);
            });
        } else {
            upkeepEl.innerHTML = '<span class="stat-item">No upkeep required</span>';
        }
    }

    // Capacity (housing / jobs / wellbeing)
    const capEl = el('inspector-capacity');
    if (capEl) {
        const parts = [];
        if (bDef.housing) parts.push(`${Math.round(bDef.housing * (1 + bonusPct / 100))} Housing`);
        if (bDef.jobs) parts.push(`${Math.round(bDef.jobs * (1 + bonusPct / 100))} Jobs`);
        if (bDef.wellbeing) parts.push(`+${(bDef.wellbeing * (1 + bonusPct / 100)).toFixed(1)} Wellbeing`);
        capEl.textContent = parts.join(' · ') || 'Standard infrastructure';
    }

    // Synergies
    const synEl = el('inspector-synergies');
    if (synEl) {
        if (bDef.active_synergies && bDef.active_synergies.length) {
            synEl.textContent = bDef.active_synergies.join(' · ');
        } else {
            synEl.textContent = 'Active with complementary tags';
        }
    }

    // Action buttons
    const btnUpgrade = el('btn-inspector-upgrade');
    const upgradeCost = el('inspector-upgrade-cost');
    if (btnUpgrade && upgradeCost) {
        if (level >= 3) {
            btnUpgrade.disabled = true;
            btnUpgrade.querySelector('.btn-main').textContent = '⭐ Maximum Level Reached';
            upgradeCost.textContent = 'Structure at peak architectural tier';
        } else {
            btnUpgrade.disabled = false;
            btnUpgrade.querySelector('.btn-main').textContent = `⭐ Upgrade to Level ${level + 1}`;
            const costStr = Object.entries(bDef.cost || {})
                .map(([r, v]) => `${Math.round(v * 0.75 * level)} ${prettyName(r)}`)
                .join(' · ');
            upgradeCost.textContent = `Cost: ${costStr} (+50% bonus)`;
        }
    }

    const btnDemolish = el('btn-inspector-demolish');
    const demolishRefund = el('inspector-demolish-refund');
    if (btnDemolish && demolishRefund) {
        btnDemolish.disabled = bDef.count <= 0;
        const refundStr = Object.entries(bDef.cost || {})
            .map(([r, v]) => `${Math.round(v * 0.40)} ${prettyName(r)}`)
            .join(' · ');
        demolishRefund.textContent = `Recycle & refund 40% (${refundStr})`;
    }

    WorldForgeCG.audio?.playBlip?.(880, 0.06);
    dialog.showModal();
}

function closeBuildingInspector() {
    const dialog = el('building-inspector-dialog');
    if (dialog && dialog.open) dialog.close();
}

function renderEcologyHud(ecology, tick = 0) {
    if (!ecology) return;

    // Season
    const seasonVal = el('hud-season-val');
    const seasonIcon = el('hud-season-icon');
    const yearLabel = el('hud-year-label');
    if (seasonVal) {
        seasonVal.textContent = ecology.season || 'Spring';
        const s = (ecology.season || '').toLowerCase();
        if (seasonIcon) {
            seasonIcon.textContent = s === 'summer' ? '☀️' : (s === 'autumn' ? '🍂' : (s === 'winter' ? '❄️' : '🌱'));
        }
    }
    if (yearLabel) {
        yearLabel.textContent = `YEAR ${ecology.year || 1}`;
    }

    // Weather & Temperature
    const weatherVal = el('hud-weather-val');
    const weatherIcon = el('hud-weather-icon');
    const tempVal = el('hud-temp-val');
    if (weatherVal) {
        weatherVal.textContent = ecology.weather || 'Clear';
        const w = (ecology.weather || '').toLowerCase();
        if (weatherIcon) {
            weatherIcon.textContent = w === 'rain' ? '🌧️' : (w === 'storm' ? '⛈️' : (w === 'drought' ? '🌫️' : (w === 'snow' ? '❄️' : '☀️')));
        }
    }
    if (tempVal) {
        tempVal.textContent = `${(ecology.temperature_c || 21.0).toFixed(1)}°C`;
    }

    // Solar Diurnal Clock
    const clockVal = el('hud-clock-val');
    const clockIcon = el('hud-clock-icon');
    const diurnalLabel = el('hud-diurnal-label');
    if (clockVal) {
        const hour = Math.floor((tick * 0.25) % 24);
        const mins = Math.floor(((tick * 0.25) % 1) * 60);
        clockVal.textContent = `${String(hour).padStart(2, '0')}:${String(mins).padStart(2, '0')}`;
        const isNight = hour >= 20 || hour < 5;
        if (clockIcon) clockIcon.textContent = isNight ? '🌙' : (hour < 8 || hour >= 18 ? '🌅' : '☀️');
        if (diurnalLabel) diurnalLabel.textContent = isNight ? 'NIGHT' : (hour < 8 ? 'DAWN' : (hour >= 18 ? 'DUSK' : 'DAY'));
    }

    // Biosphere & Pollution
    const bioVal = el('hud-bio-val');
    const polVal = el('hud-pollution-val');
    if (bioVal) {
        const bio = Math.round(ecology.biosphere_health || 95);
        bioVal.textContent = `${bio}% Bio`;
        bioVal.style.color = bio >= 75 ? 'var(--green)' : (bio >= 45 ? 'var(--amber)' : 'var(--coral)');
    }
    if (polVal) {
        const pol = Math.round(ecology.pollution || 0);
        polVal.textContent = `${pol} PPM`;
    }

    // Disaster Alert
    const disasterChip = el('hud-disaster-chip');
    const disasterVal = el('hud-disaster-val');
    if (disasterChip && disasterVal) {
        if (ecology.disaster_active) {
            disasterChip.classList.remove('hidden');
            disasterVal.textContent = ecology.disaster_name || 'Climate Shock';
        } else {
            disasterChip.classList.add('hidden');
        }
    }
}

async function toggleDefensePosture() {
    const current = state.play?.data?.city?.geopolitics?.defensePosture || 'standard';
    const next = current === 'fortified' ? 'standard' : 'fortified';
    WorldForgeCG.audio?.playServoLock?.();
    await executeGeopoliticalAction('posture', 'defense-garrison', next);
}

function openWarRoomModal() {
    const dialog = el('war-room-dialog');
    if (dialog) {
        WorldForgeCG.audio?.playWarHorn?.();
        dialog.showModal();
        if (state.play?.data?.city?.geopolitics) {
            renderGeopolitics(state.play.data.city.geopolitics);
        }
    }
}

function closeWarRoomModal() {
    const dialog = el('war-room-dialog');
    if (dialog) dialog.close();
}

function launchPlayGameMode() {
    openPlayMode();
    const microCityWorld = state.worlds.find(w => w.id === 'micro-city');
    if (microCityWorld) {
        selectWorld('micro-city');
        startPlaySession();
        WorldForgeCG.audio?.playUnlock?.();
        showToast('City Builder Online', 'Sanctuary Haven activated! Build districts and govern the realm.');
    }
}

function setupBuildDock() {
    const dock = el('city-build-dock');
    if (!dock) return;

    // Category Tabs Filtering
    const tabs = el('dock-category-tabs');
    if (tabs) {
        tabs.querySelectorAll('.dock-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                tabs.querySelectorAll('.dock-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                const cat = tab.dataset.cat;
                dock.querySelectorAll('.dock-build-btn').forEach(btn => {
                    if (cat === 'all' || btn.dataset.cat === cat) {
                        btn.classList.remove('hidden');
                    } else {
                        btn.classList.add('hidden');
                    }
                });
            });
        });
    }

    dock.querySelectorAll('.dock-build-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const bId = btn.dataset.building;
            const bDef = state.play?.data?.city?.buildings?.find(b => b.id === bId);
            if (bDef && !bDef.unlocked) {
                showToast('Technology Required', `${bDef.name} requires research: ${bDef.requiresTechnologies?.join(', ') || 'advancement'}.`, 'error');
                WorldForgeCG.audio?.playShortage?.();
                return;
            }
            if (bId) {
                if (WorldForgeCG.placementBuilding === bId) {
                    WorldForgeCG.setPlacementBuilding(null);
                } else {
                    WorldForgeCG.setPlacementBuilding(bId);
                }
            }
        });
    });

    el('btn-cancel-placement')?.addEventListener('click', () => {
        WorldForgeCG.setPlacementBuilding(null);
    });

    // Inspector dialog buttons
    el('inspector-close')?.addEventListener('click', closeBuildingInspector);
    el('btn-inspector-upgrade')?.addEventListener('click', () => {
        const dialog = el('building-inspector-dialog');
        if (dialog && dialog.dataset.building) {
            upgradeCityBuildingDirect(dialog.dataset.building);
        }
    });
    el('btn-inspector-demolish')?.addEventListener('click', () => {
        const dialog = el('building-inspector-dialog');
        if (dialog && dialog.dataset.building) {
            demolishCityBuildingDirect(dialog.dataset.building, dialog.dataset.district);
        }
    });
}

function renderRealmCard(realm) {
    const card = document.createElement('article');
    card.className = `realm-card power-${realm.powerStructure} stance-${realm.stance}`;
    
    const crests = {
        fiefdom: '🏰',
        vassal: '🌾',
        hegemon: '🦅',
        coalition: '⚖️',
        insurgency: '⚔️'
    };
    const icon = crests[realm.powerStructure] || '🛡️';

    card.innerHTML = `
        <div class="realm-card-header">
            <div class="realm-crest">${icon}</div>
            <div class="realm-identity">
                <h4>${realm.name}</h4>
                <div class="realm-tags">
                    <span class="power-badge ${realm.powerStructure}">${prettyName(realm.powerStructure)}</span>
                    <span class="stance-badge stance-${realm.stance}">${realm.stance.toUpperCase()}</span>
                </div>
            </div>
        </div>
        <p class="realm-desc">${realm.description || ''}</p>
        <div class="realm-ruler">
            <span class="ruler-title">Ruler</span>
            <strong>${realm.rulerTitle}</strong>
        </div>
        <div class="realm-meters">
            <div class="realm-meter-row">
                <span>Loyalty</span>
                <div class="meter-bar"><i style="width: ${Math.max(0, Math.min(100, realm.loyalty))}%"></i></div>
                <span>${Math.round(realm.loyalty)}%</span>
            </div>
            <div class="realm-meter-row">
                <span>Military Power</span>
                <div class="meter-bar military"><i style="width: ${Math.max(5, Math.min(100, realm.militaryPower / 3))}%"></i></div>
                <span>${Math.round(realm.militaryPower)}</span>
            </div>
        </div>
        <div class="realm-tribute-info">
            <span class="tribute-label">Tribute Offering:</span>
            <span class="tribute-amount">${Object.entries(realm.tribute || {}).map(([res, amt]) => `+${amt} ${prettyName(res)}`).join(', ') || 'None'}</span>
        </div>
    `;

    const actions = document.createElement('div');
    actions.className = 'realm-card-actions';

    if (Object.keys(realm.tribute || {}).length) {
        const btnTribute = document.createElement('button');
        btnTribute.type = 'button';
        btnTribute.className = 'button button-primary tribute-btn';
        btnTribute.textContent = '👑 Demand Tribute';
        btnTribute.disabled = !realm.tributeAvailable || state.play?.requestInFlight;
        btnTribute.addEventListener('click', () => {
            WorldForgeCG.audio?.playTribute?.();
            executeGeopoliticalAction('tribute', realm.id);
        });
        actions.append(btnTribute);
    }

    const btnEmissary = document.createElement('button');
    btnEmissary.type = 'button';
    btnEmissary.className = 'button emissary-btn';
    btnEmissary.textContent = '🕊️ Emissary (20 Cr)';
    btnEmissary.disabled = state.play?.requestInFlight;
    btnEmissary.addEventListener('click', () => {
        WorldForgeCG.audio?.playWarHorn?.();
        executeGeopoliticalAction('emissary', realm.id);
    });
    actions.append(btnEmissary);

    if (realm.stance !== 'coalition' && realm.powerStructure !== 'insurgency') {
        const btnCoalition = document.createElement('button');
        btnCoalition.type = 'button';
        btnCoalition.className = 'button coalition-btn';
        btnCoalition.textContent = '🤝 Defensive Pact';
        btnCoalition.disabled = state.play?.requestInFlight;
        btnCoalition.addEventListener('click', () => {
            WorldForgeCG.audio?.playWarHorn?.();
            executeGeopoliticalAction('coalition', realm.id);
        });
        actions.append(btnCoalition);
    }

    if (realm.raidThreat > 0 || realm.powerStructure === 'insurgency') {
        const btnStrike = document.createElement('button');
        btnStrike.type = 'button';
        btnStrike.className = 'button button-danger strike-btn';
        btnStrike.textContent = '⚔️ Preemptive Strike';
        btnStrike.disabled = state.play?.requestInFlight;
        btnStrike.addEventListener('click', () => {
            WorldForgeCG.audio?.playBattleClash?.();
            executeGeopoliticalAction('strike', realm.id);
        });
        actions.append(btnStrike);
    }

    card.append(actions);
    return card;
}

async function executeIntrigueAction(action, target, agent = null, option = null) {
    const play = state.play;
    if (!play.sessionId || play.requestInFlight || play.data?.completed) return;
    play.requestInFlight = true;
    renderCityLayer(play.data.city);
    try {
        const data = await api(`/api/play/sessions/${play.sessionId}/intrigue`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ action, target, agent, option }),
        });
        if (state.play !== play) return;
        consumePlayUpdate(data);
        const result = data.recentEvents?.find(event => event.type === 'intrigue' && event.resource !== action);
        showToast('Shadow operation resolved', result?.summary || `${prettyName(action)} executed against ${prettyName(target)}.`);
        WorldForgeCG.audio.playOverdrive();
    } catch (error) {
        showToast('Operation rejected', error.message, 'error');
    } finally {
        if (state.play !== play) return;
        play.requestInFlight = false;
        renderCityLayer(play.data?.city);
    }
}

function renderCityLayer(city) {
    const panel = el('city-layer');
    if (!panel) return;
    if (!city) {
        panel.classList.add('hidden');
        return;
    }
    panel.classList.remove('hidden');
    el('city-population').textContent = formatDecimal(city.population, 0);
    el('city-housing').textContent = formatNumber(city.housing);
    el('city-jobs').textContent = formatNumber(city.jobs);
    el('city-wellbeing').textContent = formatDecimal(city.wellbeing, 1);

    const buildingSelect = el('city-building-select');
    const currentBuilding = buildingSelect.value;
    const ids = city.buildings.map(building => building.id);
    const existing = [...buildingSelect.options].map(option => option.value);
    if (ids.join('|') !== existing.join('|')) {
        buildingSelect.replaceChildren(...city.buildings.map(building => {
            const suffix = building.unlocked ? ` · ${building.count}/${building.maxCount}` : ' · locked';
            return new Option(`${building.name}${suffix}`, building.id);
        }));
    } else {
        city.buildings.forEach((building, index) => {
            const suffix = building.unlocked ? ` · ${building.count}/${building.maxCount}` : ' · locked';
            buildingSelect.options[index].textContent = `${building.name}${suffix}`;
        });
    }
    if (ids.includes(currentBuilding)) buildingSelect.value = currentBuilding;

    // Sync RTS Quick-Build Dock buttons
    const dock = el('city-build-dock');
    if (dock && city.buildings) {
        dock.querySelectorAll('.dock-build-btn').forEach(btn => {
            const bDef = city.buildings.find(b => b.id === btn.dataset.building);
            if (bDef) {
                btn.disabled = !bDef.unlocked || !bDef.affordable;
                btn.classList.toggle('locked', !bDef.unlocked);
                btn.title = bDef.unlocked ?
                    `${bDef.name} (${bDef.count}/${bDef.maxCount}): ${bDef.description}` :
                    `${bDef.name} [LOCKED]: Requires research: ${bDef.requiresTechnologies?.join(', ') || 'advancement'}`;
            }
        });
    }

    const districts = el('city-districts');
    districts.replaceChildren(...city.districts.map(district => {
        const node = document.createElement('div'); node.className = 'district-chip';
        const label = document.createElement('span');
        const name = document.createElement('b'); name.textContent = district.name;
        const slots = document.createElement('small'); slots.textContent = `${district.usedSlots}/${district.slots}`;
        label.append(name, slots);
        const meter = document.createElement('i');
        meter.style.setProperty('--district-use', `${Math.min(100, district.usedSlots / Math.max(1, district.slots) * 100)}%`);
        node.title = district.description;
        node.append(label, meter);
        return node;
    }));

    const tree = el('city-technologies');
    const modalTree = el('modal-tree-grid');

    // Update branch counts
    const techs = city.technologies || [];
    const countAll = el('count-all'); if (countAll) countAll.textContent = techs.length;
    const countRes = el('count-resources'); if (countRes) countRes.textContent = techs.filter(t => t.branch === 'resources').length;
    const countTech = el('count-tech'); if (countTech) countTech.textContent = techs.filter(t => t.branch === 'technology').length;
    const countCult = el('count-culture'); if (countCult) countCult.textContent = techs.filter(t => t.branch === 'culture').length;
    const countEcon = el('count-economy'); if (countEcon) countEcon.textContent = techs.filter(t => t.branch === 'economy').length;
    const countSyn = el('count-synthesis'); if (countSyn) countSyn.textContent = techs.filter(t => t.branch === 'synthesis').length;

    // Filter displayed technologies
    const filteredTechs = activeTreeBranch === 'all' ? techs : techs.filter(t => t.branch === activeTreeBranch);

    if (tree) {
        tree.replaceChildren(...filteredTechs.map(createTechnologyCard));
    }
    if (modalTree) {
        modalTree.replaceChildren(...filteredTechs.map(createTechnologyCard));
    }

    updateCivilizationProgression(city);
    checkTrophies(city);

    renderCivicGovernance(
        city.governance || { factions: [], pendingDilemmas: [], decisions: [] },
        city.trajectory,
        city.systemsDebrief,
    );
    renderGeopolitics(city.geopolitics);
    renderEcologyHud(city.ecology, state.play?.data?.tick);
    renderIntrigue(city.intrigue);
    syncCityBuildingSelection();
}

function renderCivicGovernance(governance, trajectory, debrief) {
    const factions = el('civic-factions');
    factions.replaceChildren(...governance.factions.map(faction => {
        const node = document.createElement('article'); node.className = 'faction-card'; node.title = faction.description;
        const header = document.createElement('span');
        const name = document.createElement('b'); name.textContent = faction.name;
        const value = document.createElement('strong'); value.textContent = `${formatDecimal(faction.support, 0)}%`;
        header.append(name, value);
        const meter = document.createElement('i'); meter.style.setProperty('--faction-support', `${Math.max(0, Math.min(100, faction.support))}%`);
        node.append(header, meter);
        return node;
    }));

    const dilemmas = el('civic-dilemmas');
    const pending = governance.pendingDilemmas || [];
    dilemmas.replaceChildren(...(pending.length ? pending.map(dilemma => {
        const card = document.createElement('article'); card.className = 'dilemma-card';
        const head = document.createElement('div');
        const title = document.createElement('strong'); title.textContent = dilemma.title;
        const deadline = document.createElement('small');
        deadline.textContent = dilemma.deadlineTick == null ? 'Open mandate' : `Resolve by tick ${dilemma.deadlineTick}`;
        head.append(title, deadline);
        const copy = document.createElement('p'); copy.textContent = dilemma.description;
        const options = document.createElement('div'); options.className = 'dilemma-options';
        options.append(...dilemma.options.map(option => {
            const button = document.createElement('button'); button.type = 'button';
            const label = document.createElement('b'); label.textContent = option.label;
            const description = document.createElement('span'); description.textContent = option.description;
            const cost = document.createElement('small');
            const support = Object.entries(option.factionSupport || {}).map(([faction, delta]) => `${delta > 0 ? '+' : ''}${formatDecimal(delta, 0)} ${prettyName(faction)}`).join(' · ');
            const trajectoryEffects = Object.entries(option.trajectory || {}).map(([axis, delta]) => `${delta > 0 ? '↗' : '↘'} ${prettyName(axis)} ${delta > 0 ? '+' : ''}${formatDecimal(delta, 0)}`).join(' · ');
            cost.textContent = `${resourceList(option.cost)}${support ? ` · ${support}` : ''}${trajectoryEffects ? ` · ${trajectoryEffects}` : ''}`;
            button.disabled = !option.affordable || state.play.requestInFlight || state.play.data?.completed;
            button.addEventListener('click', () => makeCivicDecision(dilemma.id, option.id));
            button.append(label, description, cost);
            return button;
        }));
        card.append(head, copy, options);
        return card;
    }) : [emptyCivicState('No motion is before the council. Advance time or pursue research to reveal new dilemmas.')]));

    const history = el('civic-history');
    const decisions = governance.decisions || [];
    history.replaceChildren(...(decisions.length ? decisions.map(decision => {
        const item = document.createElement('article');
        const title = document.createElement('b'); title.textContent = decision.title;
        const choice = document.createElement('span'); choice.textContent = decision.label;
        const impact = document.createElement('small');
        const deltas = Object.entries(decision.trajectory || {})
            .map(([axis, delta]) => `${prettyName(axis)} ${delta > 0 ? '+' : ''}${formatDecimal(delta, 0)}`)
            .join(' · ');
        const counterfactuals = (decision.counterfactuals || []).map(option => option.label).join(' / ');
        impact.textContent = `${deltas || 'No direct trajectory shift'}${counterfactuals ? ` · Other path: ${counterfactuals}` : ''}`;
        impact.title = (decision.snowballingAxes || []).length
            ? `Still snowballing: ${decision.snowballingAxes.map(prettyName).join(', ')}`
            : 'Later forces are balancing this decision.';
        item.append(title, choice, impact);
        return item;
    }) : [emptyCivicState('No constitutional precedents yet.')]));

    const trajectoryNode = el('civic-trajectory');
    const scores = Object.entries(trajectory?.scores || {});
    trajectoryNode?.replaceChildren(...(scores.length ? scores
        .sort((a, b) => Math.abs(b[1]) - Math.abs(a[1]))
        .map(([axis, score]) => {
            const node = document.createElement('article'); node.className = 'faction-card';
            const header = document.createElement('span');
            const name = document.createElement('b'); name.textContent = prettyName(axis);
            const momentum = Number(trajectory?.momentum?.[axis] || 0);
            const value = document.createElement('strong');
            value.textContent = `${score >= 0 ? '+' : ''}${formatDecimal(score, 1)} ${momentum ? `(${momentum > 0 ? '↗' : '↘'}${formatDecimal(Math.abs(momentum), 1)})` : ''}`;
            header.append(name, value);
            const meter = document.createElement('i');
            meter.style.setProperty('--faction-support', `${Math.max(0, Math.min(100, 50 + score / 2))}%`);
            node.append(header, meter);
            if (axis === trajectory?.dominantAxis) {
                node.title = 'Dominant trajectory — this path currently exerts the strongest systemic feedback.';
            }
            return node;
        }) : [emptyCivicState('No trajectory established. Your first major decision will set the city in motion.')]));

    renderSystemsDebrief(debrief);
}

function renderSystemsDebrief(debrief) {
    const root = el('systems-debrief');
    if (!root) return;
    if (!debrief) {
        root.replaceChildren(emptyCivicState('Advance the simulation to reveal feedback loops, warnings, and leverage points.'));
        return;
    }
    const headline = document.createElement('p');
    headline.className = 'debrief-headline';
    headline.textContent = debrief.headline;

    const loops = document.createElement('div'); loops.className = 'debrief-loops';
    (debrief.activeFeedbackLoops || []).slice(0, 4).forEach(loop => {
        const card = document.createElement('article');
        card.className = `debrief-loop ${loop.direction === 'reinforcing' ? 'is-reinforcing' : 'is-balancing'}`;
        const heading = document.createElement('div');
        const name = document.createElement('strong'); name.textContent = prettyName(loop.axis);
        const badge = document.createElement('span'); badge.textContent = `${loop.direction} ${loop.momentum > 0 ? '↗' : '↘'}${formatDecimal(Math.abs(loop.momentum), 1)}`;
        heading.append(name, badge);
        const copy = document.createElement('p'); copy.textContent = loop.consequence;
        card.append(heading, copy); loops.append(card);
    });
    if (!loops.childElementCount) loops.append(emptyCivicState('No feedback loop has enough momentum to dominate yet.'));

    const guidance = document.createElement('div'); guidance.className = 'debrief-guidance';
    const warnings = document.createElement('article');
    const warningTitle = document.createElement('strong'); warningTitle.textContent = 'Watch next';
    const warningList = document.createElement('ul');
    (debrief.warnings?.length ? debrief.warnings : ['No critical threshold is currently breached.'])
        .forEach(value => { const item = document.createElement('li'); item.textContent = value; warningList.append(item); });
    warnings.append(warningTitle, warningList);
    const leverage = document.createElement('article');
    const leverageTitle = document.createElement('strong'); leverageTitle.textContent = 'Leverage points';
    const leverageList = document.createElement('ul');
    (debrief.leveragePoints || []).forEach(value => { const item = document.createElement('li'); item.textContent = value; leverageList.append(item); });
    leverage.append(leverageTitle, leverageList); guidance.append(warnings, leverage);
    root.replaceChildren(headline, loops, guidance);
}

function emptyCivicState(message) {
    const node = document.createElement('p'); node.className = 'civic-empty'; node.textContent = message;
    return node;
}

function renderGeopolitics(geopolitics) {
    // 1. Update 3rd World Farmer Hardship HUD
    const hud = el('city-hardship-hud');
    if (hud) {
        hud.classList.toggle('hidden', !geopolitics);
    }
    if (geopolitics) {
        const droughtVal = el('hud-drought-val');
        if (droughtVal) {
            const d = geopolitics.droughtIndex || 0;
            droughtVal.textContent = d > 50 ? 'Severe (Crit)' : (d > 25 ? 'Moderate' : 'Normal');
            droughtVal.style.color = d > 50 ? 'var(--coral)' : (d > 25 ? 'var(--amber)' : 'var(--mint)');
        }
        const famineVal = el('hud-famine-val');
        if (famineVal) {
            const f = geopolitics.famineRisk || 0;
            famineVal.textContent = f > 0.6 ? 'Crisis!' : (f > 0.25 ? 'Elevated' : 'Low');
            famineVal.style.color = f > 0.6 ? 'var(--coral)' : (f > 0.25 ? 'var(--amber)' : 'var(--mint)');
        }
        const raidVal = el('hud-raid-val');
        const threatFill = el('hud-threat-fill');
        const raidChip = el('hud-raid-chip');
        if (raidVal && threatFill) {
            const threat = Math.round(geopolitics.borderThreat || 0);
            raidVal.textContent = `${threat}%`;
            threatFill.style.width = `${Math.min(100, threat)}%`;
            if (raidChip) {
                raidChip.classList.toggle('active-raid', threat >= 70);
            }
        }
        const isFortified = geopolitics.defensePosture === 'fortified';
        const postureLabel = el('hud-posture-label');
        if (postureLabel) {
            postureLabel.textContent = isFortified ? 'Fortified Citadel' : 'Standard Garrison';
        }
        const warPosture = el('war-stat-posture');
        if (warPosture) {
            warPosture.textContent = isFortified ? 'Fortified Citadel' : 'Standard Garrison';
        }
        const warPostureBtnText = el('war-room-posture-text');
        if (warPostureBtnText) {
            warPostureBtnText.textContent = isFortified ? 'Fortified Citadel' : 'Standard Garrison';
        }
        const warThreat = el('war-stat-threat');
        if (warThreat) {
            const t = Math.round(geopolitics.borderThreat || 0);
            warThreat.textContent = `${t}% Incursion Risk`;
            warThreat.style.color = t >= 70 ? 'var(--coral)' : (t >= 35 ? 'var(--amber)' : 'var(--mint)');
        }
        const warCoalition = el('war-stat-coalition');
        if (warCoalition) {
            warCoalition.textContent = geopolitics.activeCoalition ? prettyName(geopolitics.activeCoalition) : 'None';
        }

        // War Room Entities Grid
        const grid = el('war-room-entities-grid');
        if (grid && geopolitics.entities) {
            grid.replaceChildren(...geopolitics.entities.map(renderRealmCard));
        }
    }

    const consoleNode = el('geopolitics-console');
    if (!consoleNode) return;
    consoleNode.classList.toggle('hidden', !geopolitics);
    if (!geopolitics) return;
    const stats = el('geopolitics-stats');
    if (stats) {
        stats.replaceChildren(
            strategyStat('Defense', geopolitics.defensePosture),
            strategyStat('Power', formatDecimal(geopolitics.militaryPower, 0)),
            strategyStat('Border threat', `${formatDecimal(geopolitics.borderThreat, 0)}%`),
            strategyStat('Famine risk', `${formatDecimal((geopolitics.famineRisk || 0) * 100, 0)}%`),
        );
    }
    const realms = el('geopolitics-entities');
    if (realms && geopolitics.entities) {
        realms.replaceChildren(...geopolitics.entities.map(realm => {
            const card = document.createElement('article'); card.className = 'realm-card';
            const head = document.createElement('div');
            const title = document.createElement('strong'); title.textContent = realm.name;
            const stance = document.createElement('span'); stance.className = `stance stance-${realm.stance}`; stance.textContent = realm.stance;
            head.append(title, stance);
            const copy = document.createElement('p'); copy.textContent = `${realm.rulerTitle} · ${prettyName(realm.powerStructure)} · loyalty ${formatDecimal(realm.loyalty, 0)} · power ${formatDecimal(realm.militaryPower, 0)}`;
            const actions = document.createElement('div'); actions.className = 'mini-actions';
            if (Object.keys(realm.tribute || {}).length) actions.append(strategyButton('Tribute', () => executeGeopoliticalAction('tribute', realm.id), !realm.tributeAvailable));
            actions.append(strategyButton('Emissary', () => executeGeopoliticalAction('emissary', realm.id)));
            if (realm.stance !== 'coalition') actions.append(strategyButton('Coalition', () => executeGeopoliticalAction('coalition', realm.id)));
            if (realm.raidThreat > 0) actions.append(strategyButton('Strike', () => executeGeopoliticalAction('strike', realm.id), false, 'danger'));
            card.append(head, copy, actions);
            return card;
        }));
    }
}

function renderIntrigue(intrigue) {
    const consoleNode = el('intrigue-console');
    const marketNode = el('crypto-console');
    if (!consoleNode || !marketNode) return;
    consoleNode.classList.toggle('hidden', !intrigue);
    marketNode.classList.toggle('hidden', !intrigue);
    if (!intrigue) return;
    el('intrigue-heat').textContent = formatDecimal(intrigue.heat, 0);
    el('intrigue-heat').style.setProperty('--heat', `${Math.max(0, Math.min(100, intrigue.heat))}%`);
    const readyAgent = intrigue.agents.find(agent => agent.status === 'ready');

    el('intrigue-corporations').replaceChildren(...intrigue.corporations.map(corporation => {
        const card = document.createElement('article'); card.className = 'corporation-card';
        const head = document.createElement('div');
        const title = document.createElement('strong'); title.textContent = corporation.name;
        const sector = document.createElement('span'); sector.textContent = corporation.sector;
        head.append(title, sector);
        const metrics = document.createElement('p'); metrics.textContent = `Security ${formatDecimal(corporation.security, 0)} · influence ${formatDecimal(corporation.influence, 0)} · exposure ${formatDecimal(corporation.exposure, 0)} · ${corporation.remainingSecrets} secrets`;
        const actions = document.createElement('div'); actions.className = 'mini-actions';
        actions.append(strategyButton('Infiltrate', () => executeIntrigueAction('infiltrate', corporation.id, readyAgent?.id), !readyAgent || corporation.remainingSecrets === 0));
        actions.append(strategyButton('Counterintel', () => executeIntrigueAction('counterintel', corporation.id)));
        card.append(head, metrics, actions);
        return card;
    }));

    el('intrigue-agents').replaceChildren(...intrigue.agents.map(agent => {
        const card = document.createElement('article'); card.className = `agent-card agent-${agent.status}`;
        const head = document.createElement('div');
        const title = document.createElement('strong'); title.textContent = agent.name;
        const status = document.createElement('span'); status.textContent = agent.status;
        head.append(title, status);
        const metrics = document.createElement('p'); metrics.textContent = `Skill ${formatDecimal(agent.skill, 0)} · stealth ${formatDecimal(agent.stealth, 0)} · loyalty ${formatDecimal(agent.loyalty, 0)} · containment ${formatDecimal(agent.containment, 0)}`;
        card.append(head, metrics);
        if (agent.status === 'contained' || agent.status === 'compromised') card.append(strategyButton('Audit & deploy', () => executeIntrigueAction('deploy', agent.id)));
        if (agent.status === 'rogue' || agent.status === 'compromised') card.append(strategyButton('Contain', () => executeIntrigueAction('contain', agent.id), false, 'danger'));
        return card;
    }));

    el('intrigue-markets').replaceChildren(...intrigue.markets.map(market => {
        const card = document.createElement('article'); card.className = 'market-card';
        const head = document.createElement('div');
        const name = document.createElement('strong'); name.textContent = market.symbol;
        const price = document.createElement('b'); price.textContent = `${formatDecimal(market.price, 2)} cr`;
        head.append(name, price);
        const change = document.createElement('p'); change.className = market.changePercent >= 0 ? 'positive' : 'negative'; change.textContent = `${market.changePercent >= 0 ? '+' : ''}${formatDecimal(market.changePercent, 1)}% · position ${formatDecimal(market.positionValue, 1)} cr`;
        const actions = document.createElement('div'); actions.className = 'mini-actions';
        actions.append(strategyButton('Buy', () => executeIntrigueAction('trade', market.id, null, 'buy')));
        actions.append(strategyButton('Sell', () => executeIntrigueAction('trade', market.id, null, 'sell'), market.holdings <= 0));
        actions.append(strategyButton('Pump', () => executeIntrigueAction('manipulate', market.id, readyAgent?.id, 'pump'), false, 'shadow'));
        actions.append(strategyButton('Dump', () => executeIntrigueAction('manipulate', market.id, readyAgent?.id, 'dump'), false, 'shadow'));
        card.append(head, change, actions);
        return card;
    }));

    const secrets = intrigue.stolenSecrets || [];
    el('intrigue-secrets').replaceChildren(...(secrets.length ? secrets.map(secret => {
        const node = document.createElement('article');
        const name = document.createElement('strong'); name.textContent = secret.name;
        const source = document.createElement('span'); source.textContent = `${prettyName(secret.corporation)} · +${formatDecimal(secret.researchValue, 0)} research`;
        node.append(name, source);
        return node;
    }) : [emptyCivicState('No trade secrets acquired.') ]));
}

function strategyStat(label, value) {
    const node = document.createElement('span');
    const small = document.createElement('small'); small.textContent = label;
    const strong = document.createElement('strong'); strong.textContent = value;
    node.append(small, strong); return node;
}

function strategyButton(label, handler, disabled = false, tone = '') {
    const button = document.createElement('button'); button.type = 'button'; button.textContent = label;
    button.className = tone ? `tone-${tone}` : '';
    button.disabled = disabled || state.play.requestInFlight || state.play.data?.completed;
    button.addEventListener('click', handler);
    return button;
}

function syncCityBuildingSelection() {
    const city = state.play.data?.city;
    if (!city) return;
    const building = city.buildings.find(item => item.id === el('city-building-select').value) || city.buildings[0];
    if (!building) return;
    el('city-building-select').value = building.id;
    const districtSelect = el('city-district-select');
    const currentDistrict = districtSelect.value;
    const allowed = city.districts.filter(district => building.allowedDistricts.includes(district.id));
    districtSelect.replaceChildren(...allowed.map(district => new Option(`${district.name} · ${district.slots - district.usedSlots} free`, district.id)));
    if (allowed.some(district => district.id === currentDistrict)) districtSelect.value = currentDistrict;
    const district = allowed.find(item => item.id === districtSelect.value);

    const detail = el('city-building-detail'); detail.replaceChildren();
    const title = document.createElement('strong'); title.textContent = `${building.name} · ${prettyName(building.category || 'civic')}`;
    const copy = document.createElement('span'); copy.textContent = building.description;
    const economy = document.createElement('span'); economy.className = 'city-cost';
    economy.textContent = `Build ${resourceList(building.cost)} · upkeep ${resourceList(building.upkeep)} · yields ${resourceList(building.outputs)}`;
    const impact = document.createElement('span');
    impact.textContent = `Housing +${building.housing} · jobs +${building.jobs} · wellbeing +${formatDecimal(building.wellbeing, 1)}${building.requiresTechnologies.length ? ` · requires ${building.requiresTechnologies.map(prettyName).join(', ')}` : ''}`;
    detail.append(title, copy, document.createElement('br'), economy, document.createElement('br'), impact);

    const hasSpace = district && district.usedSlots + building.footprint <= district.slots;
    el('city-build-button').disabled = !building.unlocked || !building.affordable || building.count >= building.maxCount || !hasSpace || state.play.requestInFlight || state.play.data?.completed;
}

function resourceList(resources) {
    const entries = Object.entries(resources || {});
    return entries.length ? entries.map(([resource, amount]) => `${formatDecimal(amount, 1)} ${prettyName(resource)}`).join(' + ') : 'none';
}

function updateCapacityLabel() {
    const val = el('capacity-slider').value;
    el('capacity-value').textContent = `${val}%`;
    syncPresetHighlight(val);
}

function syncCapacityControl() {
    const entity = state.play.data?.entityStates.find(item => item.name === el('play-entity-select').value);
    if (!entity || entity.capacity == null) return;
    const val = Math.round(entity.capacity * 100);
    el('capacity-slider').value = val;
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
    renderCityLayer(data.city);
    updatePlayStatus();
    updateVitalityGauge(data);
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
    WorldForgeCG.updateAvatarHero(select.value);
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

function escapeHtml(text) {
    if (!text) return '';
    return String(text)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}

function openAnalytics() {
    if (!currentWorld()) return;
    el('analytics-runs').value = '15';
    el('analytics-ticks').value = dom.ticks.value || '500';
    el('analytics-seed').value = dom.seed.value || '42';
    dom.analyticsDialog.showModal();
    if (state.analytics.report && state.analytics.report.world === state.worldId) {
        renderAnalytics(state.analytics.report);
    }
}

async function runAnalytics() {
    let runs, ticks, seed;
    try {
        runs = numericInput(el('analytics-runs'), { min: 2, max: 100, name: 'Monte Carlo runs' });
        ticks = numericInput(el('analytics-ticks'), { min: 50, max: 10000, name: 'Ticks per run' });
        seed = numericInput(el('analytics-seed'), { min: 0, max: Number.MAX_SAFE_INTEGER, name: 'Base seed' });
    } catch (error) {
        showToast('Check Risk Lab settings', error.message, 'error');
        return;
    }
    const button = el('analytics-run-btn');
    setButtonBusy(button, true, 'Running Monte Carlo…');
    const results = el('analytics-results');
    results.replaceChildren(emptyState('◌', 'Monte Carlo sweep in progress', `Simulating ${runs} seeds across ${formatNumber(ticks)} ticks on the deterministic engine.`));
    try {
        const report = await api('/api/analyze', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ world: state.worldId, runs, ticks, seed }),
        });
        state.analytics.report = report;
        const resources = Object.keys(report.confidenceBands || {});
        state.analytics.activeResource = resources[0] || null;
        renderAnalytics(report);
        showToast('Monte Carlo sweep complete', `${runs} runs evaluated. Risk profile: ${report.riskLevel}.`);
    } catch (error) {
        results.replaceChildren(emptyState('!', 'Analysis failed', error.message));
    } finally {
        setButtonBusy(button, false, 'Execute Monte Carlo');
    }
}

function drawFanChart(report, resourceName) {
    if (!report || !report.confidenceBands) return;
    const bands = report.confidenceBands[resourceName];
    if (!bands || !bands.length) return;

    const { ctx, width: w, height: h } = prepareCanvas('analytics-fan-chart', 260);
    const pad = { top: 32, right: 24, bottom: 36, left: 56 };
    const cw = w - pad.left - pad.right;
    const ch = h - pad.top - pad.bottom;

    let minVal = Infinity, maxVal = -Infinity;
    let minTick = bands[0].tick, maxTick = bands[bands.length - 1].tick;

    bands.forEach(pt => {
        if (pt.min < minVal) minVal = pt.min;
        if (pt.max > maxVal) maxVal = pt.max;
    });

    if (minVal > 0) minVal = 0;
    if (maxVal <= minVal) maxVal = minVal + 1;
    const valRange = maxVal - minVal;
    const tickRange = Math.max(1, maxTick - minTick);

    const getX = tick => pad.left + ((tick - minTick) / tickRange) * cw;
    const getY = val => pad.top + ch - ((val - minVal) / valRange) * ch;

    // Gridlines & Y-axis numbers
    ctx.lineWidth = 1;
    ctx.font = '10px ' + FONT_MONO;
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = 'rgba(163, 184, 214, 0.45)';

    const steps = 4;
    for (let i = 0; i <= steps; i++) {
        const val = minVal + (valRange * i) / steps;
        const y = pad.top + ch - (ch * i) / steps;
        ctx.strokeStyle = 'rgba(163, 184, 214, 0.08)';
        ctx.beginPath();
        ctx.moveTo(pad.left, y);
        ctx.lineTo(pad.left + cw, y);
        ctx.stroke();

        ctx.fillText(compactNumber(val), pad.left - 8, y);
    }

    // X-axis tick labels
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    const xSteps = Math.min(6, bands.length);
    for (let i = 0; i < xSteps; i++) {
        const idx = Math.floor((i / (xSteps - 1)) * (bands.length - 1));
        const pt = bands[idx];
        const x = getX(pt.tick);
        ctx.fillText(`t=${pt.tick}`, x, pad.top + ch + 8);
    }

    // 1. Draw p10 - p90 area (80% confidence envelope)
    ctx.beginPath();
    ctx.moveTo(getX(bands[0].tick), getY(bands[0].p90));
    for (let i = 1; i < bands.length; i++) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].p90));
    }
    for (let i = bands.length - 1; i >= 0; i--) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].p10));
    }
    ctx.closePath();
    ctx.fillStyle = 'rgba(103, 169, 255, 0.16)';
    ctx.fill();

    // 2. Draw p25 - p75 area (50% confidence envelope)
    ctx.beginPath();
    ctx.moveTo(getX(bands[0].tick), getY(bands[0].p75));
    for (let i = 1; i < bands.length; i++) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].p75));
    }
    for (let i = bands.length - 1; i >= 0; i--) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].p25));
    }
    ctx.closePath();
    ctx.fillStyle = 'rgba(103, 169, 255, 0.35)';
    ctx.fill();

    // 3. Draw min/max dotted boundaries
    ctx.save();
    ctx.setLineDash([2, 3]);
    ctx.strokeStyle = 'rgba(163, 184, 214, 0.35)';
    ctx.lineWidth = 1;

    ctx.beginPath();
    ctx.moveTo(getX(bands[0].tick), getY(bands[0].max));
    for (let i = 1; i < bands.length; i++) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].max));
    }
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(getX(bands[0].tick), getY(bands[0].min));
    for (let i = 1; i < bands.length; i++) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].min));
    }
    ctx.stroke();
    ctx.restore();

    // 4. Draw median solid line
    ctx.beginPath();
    ctx.moveTo(getX(bands[0].tick), getY(bands[0].median));
    for (let i = 1; i < bands.length; i++) {
        ctx.lineTo(getX(bands[i].tick), getY(bands[i].median));
    }
    ctx.strokeStyle = '#42d3ea';
    ctx.lineWidth = 2.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.stroke();

    // 5. Draw active resource badge and current median
    const lastPt = bands[bands.length - 1];
    ctx.textAlign = 'left';
    ctx.textBaseline = 'top';
    ctx.fillStyle = '#67a9ff';
    ctx.font = '700 11px ' + FONT_MONO;
    ctx.fillText(`${resourceName.toUpperCase()} · Median: ${lastPt.median.toFixed(1)} [p10: ${lastPt.p10.toFixed(1)}, p90: ${lastPt.p90.toFixed(1)}]`, pad.left + 4, 10);
}

function renderAnalytics(report) {
    const results = el('analytics-results');
    results.replaceChildren();

    const world = currentWorld();
    const heading = document.createElement('div');
    heading.className = 'result-heading';
    const title = document.createElement('strong');
    title.textContent = `${world?.title || report.world} · Monte Carlo Risk Report`;
    const detail = document.createElement('span');
    detail.textContent = `${report.runs} stochastic runs · ${formatNumber(report.ticksPerRun)} ticks each · seeds ${report.seeds[0]}..=${report.seeds[report.seeds.length - 1]}`;
    heading.append(title, detail);

    // Headline KPIs
    const kpis = document.createElement('div');
    kpis.className = 'analytics-kpis';

    const resDist = report.resilienceDistribution;
    const kpiResilience = document.createElement('div');
    kpiResilience.className = 'analytics-kpi';
    kpiResilience.innerHTML = `<span>Resilience Score</span><strong>${resDist.mean.toFixed(1)} <small>(±${resDist.stdDev.toFixed(1)})</small></strong><small>Median: ${resDist.median.toFixed(1)} · p10–p90: ${resDist.p10.toFixed(0)}–${resDist.p90.toFixed(0)}</small>`;

    const kpiRisk = document.createElement('div');
    kpiRisk.className = 'analytics-kpi';
    const riskClass = report.riskLevel.toLowerCase();
    kpiRisk.innerHTML = `<span>Risk Profile</span><strong><span class="risk-badge ${riskClass}">${report.riskLevel}</span></strong><small>${report.blackSwanRuns} shock runs &lt;35/100</small>`;

    const passedObjs = Object.values(report.objectiveSuccessRates).filter(r => r >= 0.95).length;
    const totalObjs = Object.keys(report.objectiveSuccessRates).length;
    const kpiObjectives = document.createElement('div');
    kpiObjectives.className = 'analytics-kpi';
    kpiObjectives.innerHTML = `<span>Objective Resilience</span><strong>${passedObjs} / ${totalObjs} Robust</strong><small>${((passedObjs/Math.max(1, totalObjs))*100).toFixed(0)}% scenarios secure</small>`;

    const topBottleneck = report.bottlenecks[0];
    const kpiBottleneck = document.createElement('div');
    kpiBottleneck.className = 'analytics-kpi';
    kpiBottleneck.innerHTML = `<span>Primary Constraint</span><strong>${topBottleneck ? prettyName(topBottleneck.entity) : 'None'}</strong><small>${topBottleneck ? 'Score ' + topBottleneck.bottleneckScore.toFixed(1) : 'Flow optimal'}</small>`;

    kpis.append(kpiResilience, kpiRisk, kpiObjectives, kpiBottleneck);

    // Primary Vulnerability banner
    const alert = document.createElement('div');
    alert.className = `analytics-vulnerability-alert ${riskClass}`;
    alert.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m12 3 9 17H3L12 3Zm0 6v5m0 3v.1" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg><div><strong>Diagnosis:</strong> ${escapeHtml(report.primaryVulnerability)}</div>`;

    // Fan Chart Section
    const chartSection = document.createElement('div');
    chartSection.className = 'analytics-section';
    const chartHead = document.createElement('div');
    chartHead.className = 'analytics-section-head';
    chartHead.innerHTML = `<h3>Stochastic Confidence Envelopes</h3>`;

    const chartControls = document.createElement('div');
    chartControls.className = 'fan-chart-controls';
    const select = document.createElement('select');
    select.id = 'fan-resource-select';
    select.className = 'field';
    const resKeys = Object.keys(report.confidenceBands || {});
    resKeys.forEach(res => {
        const opt = document.createElement('option');
        opt.value = res;
        opt.textContent = `${prettyName(res)}`;
        if (res === state.analytics.activeResource) opt.selected = true;
        select.append(opt);
    });
    select.addEventListener('change', () => {
        state.analytics.activeResource = select.value;
        drawFanChart(report, select.value);
    });
    chartControls.append(select);
    chartHead.append(chartControls);

    const canvasWrap = document.createElement('div');
    canvasWrap.className = 'fan-chart-wrap';
    const canvas = document.createElement('canvas');
    canvas.id = 'analytics-fan-chart';
    canvasWrap.append(canvas);

    const legend = document.createElement('div');
    legend.className = 'fan-chart-legend';
    legend.innerHTML = `
        <span><i class="fan-legend-swatch fan-swatch-p90"></i> p10–p90 (80% Envelope)</span>
        <span><i class="fan-legend-swatch fan-swatch-p75"></i> p25–p75 (50% Envelope)</span>
        <span><i class="fan-legend-swatch fan-swatch-median"></i> Median Path</span>
        <span><i class="fan-legend-swatch fan-swatch-minmax"></i> Min–Max Bounds</span>
    `;
    chartSection.append(chartHead, canvasWrap, legend);

    // Objective Success Rates
    const objSection = document.createElement('div');
    objSection.className = 'analytics-section';
    objSection.innerHTML = `<div class="analytics-section-head"><h3>Scenario Objectives Success Probability</h3><span>Across ${report.runs} independent seeds</span></div>`;
    const objGrid = document.createElement('div');
    objGrid.className = 'analytics-objectives-grid';
    for (const [name, rate] of Object.entries(report.objectiveSuccessRates)) {
        const pct = Math.round(rate * 100);
        const card = document.createElement('div');
        card.className = 'analytics-objective-card';
        const color = pct >= 95 ? 'var(--green)' : pct >= 70 ? 'var(--blue)' : 'var(--red)';
        card.innerHTML = `
            <div class="analytics-obj-head"><span>${escapeHtml(name)}</span><strong style="color: ${color}">${pct}%</strong></div>
            <div class="analytics-obj-bar"><div class="analytics-obj-fill" style="width: ${pct}%; background: ${color};"></div></div>
            <div class="analytics-obj-meta"><span>${pct >= 95 ? 'Robust' : pct >= 70 ? 'Moderate' : 'Fragile'}</span><span>${pct === 100 ? 'Zero failures' : (100 - pct) + '% failure rate'}</span></div>
        `;
        objGrid.append(card);
    }
    objSection.append(objGrid);

    // Bottlenecks Section
    const bSection = document.createElement('div');
    bSection.className = 'analytics-section';
    bSection.innerHTML = `<div class="analytics-section-head"><h3>Systemic Bottleneck Diagnosis</h3><span>Ranked by friction &amp; starvation impact</span></div>`;
    const bList = document.createElement('div');
    bList.className = 'bottleneck-rank-list';
    report.bottlenecks.forEach((b, idx) => {
        const row = document.createElement('div');
        row.className = 'bottleneck-row';
        const rankClass = idx === 0 ? 'rank-1' : idx === 1 ? 'rank-2' : 'rank-other';
        const scorePct = Math.min(100, Math.round(b.bottleneckScore * 10));
        const barColor = b.bottleneckScore >= 20 ? 'var(--red)' : b.bottleneckScore >= 8 ? 'var(--amber)' : 'var(--green)';
        row.innerHTML = `
            <span class="bottleneck-rank ${rankClass}">#${idx + 1}</span>
            <div class="bottleneck-entity"><strong>${prettyName(b.entity)}</strong><small>${b.shortageCount} shortages · ${(b.starvationRatio * 100).toFixed(0)}% starved</small></div>
            <div class="bottleneck-desc">${escapeHtml(b.impactSummary)}</div>
            <div class="bottleneck-meter-wrap">
                <strong style="color: ${barColor}">${b.bottleneckScore.toFixed(1)}</strong>
                <div class="bottleneck-bar"><div class="bottleneck-fill" style="width: ${scorePct}%; background: ${barColor}"></div></div>
            </div>
        `;
        bList.append(row);
    });
    bSection.append(bList);

    // Tables Split: Resource Elasticity + Link Elasticity
    const tablesSection = document.createElement('div');
    tablesSection.className = 'analytics-section';
    tablesSection.innerHTML = `<div class="analytics-section-head"><h3>Flow Elasticity &amp; Capacity Margins</h3></div>`;
    const tablesSplit = document.createElement('div');
    tablesSplit.className = 'analytics-tables-split';

    // Resource Buffer Elasticity Table
    const resWrap = document.createElement('div');
    resWrap.className = 'analytics-table-wrap';
    resWrap.innerHTML = `
        <table class="analytics-table">
            <thead><tr><th>Resource</th><th class="num">Burn / t</th><th class="num">Replenish</th><th class="num">Buffer Runway</th></tr></thead>
            <tbody>
                ${report.resourceElasticity.map(r => `
                    <tr>
                        <td><strong>${prettyName(r.resource)}</strong></td>
                        <td class="num">${r.burnRate.toFixed(2)}</td>
                        <td class="num" style="color: ${r.replenishmentRatio >= 1.0 ? 'var(--green)' : 'var(--amber)'}">${r.replenishmentRatio.toFixed(2)}x</td>
                        <td class="num">${r.bufferRunwayTicks >= 9999 ? '∞' : r.bufferRunwayTicks.toFixed(0) + ' t'}</td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;

    // Link Elasticity Table
    const linkWrap = document.createElement('div');
    linkWrap.className = 'analytics-table-wrap';
    linkWrap.innerHTML = `
        <table class="analytics-table">
            <thead><tr><th>Conduit</th><th>Res</th><th class="num">Transferred</th><th class="num">Congestion</th></tr></thead>
            <tbody>
                ${report.linkElasticity.map(l => `
                    <tr>
                        <td><strong>${prettyName(l.from)} → ${prettyName(l.to)}</strong></td>
                        <td>${prettyName(l.resource)}</td>
                        <td class="num">${formatNumber(l.totalTransferred)} / ${formatNumber(l.maxCapacity)}</td>
                        <td class="num" style="color: ${l.congestionRatio > 0.3 ? 'var(--red)' : l.congestionRatio > 0.1 ? 'var(--amber)' : 'var(--text-muted)'}">${(l.congestionRatio * 100).toFixed(0)}%</td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;
    tablesSplit.append(resWrap, linkWrap);
    tablesSection.append(tablesSplit);

    // Cross-Resource Pearson Correlation Matrix Heatmap
    const corrSection = document.createElement('div');
    corrSection.className = 'analytics-section';
    corrSection.innerHTML = `<div class="analytics-section-head"><h3>Cross-Resource Pearson Correlation Matrix</h3><span>Inverse (&lt;0) vs coupled (&gt;0) systemic dependencies</span></div>`;

    const matrix = report.correlationMatrix;
    const resources = Object.keys(matrix);
    const corrWrap = document.createElement('div');
    corrWrap.className = 'analytics-table-wrap';

    let tableHtml = `<table class="correlation-matrix-table"><thead><tr><th></th>${resources.map(r => `<th>${prettyName(r)}</th>`).join('')}</tr></thead><tbody>`;
    resources.forEach(rA => {
        tableHtml += `<tr><th>${prettyName(rA)}</th>`;
        resources.forEach(rB => {
            const val = matrix[rA]?.[rB] ?? 0;
            let bgColor = 'rgba(255,255,255,0.02)';
            let textColor = 'var(--text-muted)';
            if (val > 0.2) {
                bgColor = `rgba(66, 211, 234, ${Math.min(0.6, val * 0.5)})`;
                textColor = 'var(--cyan)';
            } else if (val < -0.2) {
                bgColor = `rgba(255, 111, 124, ${Math.min(0.6, Math.abs(val) * 0.5)})`;
                textColor = 'var(--red)';
            }
            tableHtml += `<td><div class="correlation-cell" style="background: ${bgColor}; color: ${textColor}">${val.toFixed(2)}</div></td>`;
        });
        tableHtml += `</tr>`;
    });
    tableHtml += `</tbody></table>`;
    corrWrap.innerHTML = tableHtml;
    corrSection.append(corrWrap);

    results.append(heading, kpis, alert, chartSection, objSection, bSection, tablesSection, corrSection);

    // Initial Fan Chart render
    const activeRes = state.analytics.activeResource || resKeys[0];
    if (activeRes) {
        setTimeout(() => drawFanChart(report, activeRes), 20);
    }
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
    ctx.font = '10px ' + FONT_MONO;
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
    ctx.font = '10px ' + FONT_SANS;
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
            ctx.fillStyle = '#9aa8bb'; ctx.font = '9px ' + FONT_SANS; ctx.textAlign = 'center';
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
        ctx.fillStyle = '#b7c1cf'; ctx.font = '10px ' + FONT_MONO; ctx.textAlign = 'center'; ctx.fillText(compactNumber(count), x + barWidth / 2, y - 8);
        ctx.fillStyle = '#718097'; ctx.font = '9px ' + FONT_SANS; ctx.fillText(prettyName(type), x + barWidth / 2, h - 16);
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
    if (type.includes('water') || name.includes('water') || name.includes('river') || name.includes('ocean') || name.includes('sea') || name.includes('aquifer')) return 'water';
    if (type.includes('predator') || name.includes('wolf') || name.includes('carnivore')) return 'predator';
    if (type.includes('prey') || name.includes('deer') || name.includes('herd') || name.includes('wildlife') || name.includes('fauna')) return 'wildlife';
    if (name.includes('grass') || name.includes('forest') || name.includes('vegetation') || name.includes('flora') || name.includes('biome') || type.includes('ecosystem') || type.includes('bio') || type.includes('food') || name.includes('farm')) return 'bio';
    if (type.includes('renewable') || type.includes('power') || name.includes('wind') || name.includes('solar') || name.includes('sunlight') || name.includes('energy')) return 'power';
    if (type.includes('extractor') || name.includes('mine') || name.includes('quarry') || name.includes('well')) return 'extractor';
    if (type.includes('processor') || type.includes('refiner') || name.includes('mill') || name.includes('chemical') || name.includes('desal')) return 'processor';
    if (type.includes('manufacturer') || name.includes('factory') || name.includes('assembly')) return 'manufacturer';
    if (type.includes('distributor') || name.includes('warehouse') || name.includes('depot') || name.includes('port') || name.includes('storage')) return 'depot';
    if (type.includes('consumer') || name.includes('city') || name.includes('market') || name.includes('population') || name.includes('resident')) return 'habitat';
    if (type.includes('care') || name.includes('medical') || name.includes('hospital') || name.includes('clinic')) return 'care';
    if (type.includes('producer')) return 'extractor';
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
    wildlife: { color: '#fbbf24', label: 'HERBIVORE' },
    predator: { color: '#f43f5e', label: 'PREDATOR' },
    water: { color: '#06b6d4', label: 'HYDRO' },
    care: { color: '#ec4899', label: 'MEDICAL' },
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
    perspectiveMode: 'strategic',
    worldLens: 'city',
    pressedKeys: new Set(),
    walker: { x: 0, y: 0, heading: 0, bob: 0 },
    immersionPulse: { color: '#42d3ea', alpha: 0, label: '' },
    placementBuilding: null,
    hoveredTile: null,
    hoveredDistrict: null,
    hoveredRealmEntity: null,
    cityCitizens: [],
    chimneySmoke: [],
    cityLifeInit: false,
    hasChosenLens: false,

    data: null,
    nodes: new Map(),
    links: [],
    particles: [],
    shockwaves: [],
    floaties: [],
    ambientDust: [],
    envWeather: [],
    envWeatherType: 'cyber',

    audio: {
        ctx: null,
        enabled: localStorage.getItem('worldforge_sfx') !== 'false',
        init() {
            if (!this.ctx && typeof AudioContext !== 'undefined') {
                try {
                    this.ctx = new (window.AudioContext || window.webkitAudioContext)();
                } catch (_) {}
            }
            if (this.ctx && this.ctx.state === 'suspended') {
                this.ctx.resume().catch(() => {});
            }
        },
        toggle() {
            this.enabled = !this.enabled;
            localStorage.setItem('worldforge_sfx', String(this.enabled));
            if (this.enabled) this.init();
            return this.enabled;
        },
        playBlip(freq = 640, duration = 0.06, type = 'sine') {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                const osc = this.ctx.createOscillator();
                const gain = this.ctx.createGain();
                osc.type = type;
                osc.frequency.setValueAtTime(freq, now);
                osc.frequency.exponentialRampToValueAtTime(freq * 1.5, now + duration);
                gain.gain.setValueAtTime(0.08, now);
                gain.gain.exponentialRampToValueAtTime(0.001, now + duration);
                osc.connect(gain);
                gain.connect(this.ctx.destination);
                osc.start(now);
                osc.stop(now + duration);
            } catch (_) {}
        },
        playShortage() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                [420, 310].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sawtooth';
                    osc.frequency.setValueAtTime(freq, now + i * 0.08);
                    gain.gain.setValueAtTime(0.06, now + i * 0.08);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.08 + 0.12);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.08);
                    osc.stop(now + i * 0.08 + 0.12);
                });
            } catch (_) {}
        },
        playOverdrive() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                const osc = this.ctx.createOscillator();
                const gain = this.ctx.createGain();
                osc.type = 'triangle';
                osc.frequency.setValueAtTime(340, now);
                osc.frequency.exponentialRampToValueAtTime(880, now + 0.15);
                gain.gain.setValueAtTime(0.09, now);
                gain.gain.exponentialRampToValueAtTime(0.001, now + 0.15);
                osc.connect(gain);
                gain.connect(this.ctx.destination);
                osc.start(now);
                osc.stop(now + 0.15);
            } catch (_) {}
        },
        playUnlock() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Resonant ascending major chord fanfare: C5 (523), E5 (659), G5 (784), C6 (1046)
                [523.25, 659.25, 783.99, 1046.5].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'triangle';
                    osc.frequency.setValueAtTime(freq, now + i * 0.07);
                    gain.gain.setValueAtTime(0.09, now + i * 0.07);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.07 + 0.35);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.07);
                    osc.stop(now + i * 0.07 + 0.35);
                });
            } catch (_) {}
        },
        playConstruction() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                const osc = this.ctx.createOscillator();
                const gain = this.ctx.createGain();
                osc.type = 'sawtooth';
                osc.frequency.setValueAtTime(140, now);
                osc.frequency.exponentialRampToValueAtTime(45, now + 0.12);
                gain.gain.setValueAtTime(0.12, now);
                gain.gain.exponentialRampToValueAtTime(0.001, now + 0.12);
                osc.connect(gain);
                gain.connect(this.ctx.destination);
                osc.start(now);
                osc.stop(now + 0.12);
            } catch (_) {}
        },
        playLevelUp() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Celebratory fanfare
                [440, 554.37, 659.25, 880].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(freq, now + i * 0.09);
                    gain.gain.setValueAtTime(0.1, now + i * 0.09);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.09 + 0.45);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.09);
                    osc.stop(now + i * 0.09 + 0.45);
                });
            } catch (_) {}
        },
        playTrophy() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                [784, 987.77, 1174.66].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(freq, now + i * 0.08);
                    gain.gain.setValueAtTime(0.07, now + i * 0.08);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.08 + 0.3);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.08);
                    osc.stop(now + i * 0.08 + 0.3);
                });
            } catch (_) {}
        },
        playChaos() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                [660, 480, 320, 220].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sawtooth';
                    osc.frequency.setValueAtTime(freq, now + i * 0.1);
                    gain.gain.setValueAtTime(0.08, now + i * 0.1);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.1 + 0.2);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.1);
                    osc.stop(now + i * 0.1 + 0.2);
                });
            } catch (_) {}
        },
        playWarHorn() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Deep resonant medieval/fantasy war horn (sawtooth + bandpass drone)
                [110, 146.83, 164.81].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    const filter = this.ctx.createBiquadFilter();
                    osc.type = 'sawtooth';
                    filter.type = 'lowpass';
                    filter.frequency.setValueAtTime(450, now);
                    filter.frequency.exponentialRampToValueAtTime(800, now + 0.2);
                    filter.frequency.exponentialRampToValueAtTime(300, now + 0.6);
                    osc.frequency.setValueAtTime(freq, now);
                    osc.frequency.setValueAtTime(freq * 0.98, now + 0.3);
                    gain.gain.setValueAtTime(0.001, now);
                    gain.gain.linearRampToValueAtTime(0.12, now + 0.08);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.7);
                    osc.connect(filter);
                    filter.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now);
                    osc.stop(now + 0.7);
                });
            } catch (_) {}
        },
        playBattleClash() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Sharp metallic strike + sword clang
                const osc = this.ctx.createOscillator();
                const gain = this.ctx.createGain();
                osc.type = 'triangle';
                osc.frequency.setValueAtTime(980, now);
                osc.frequency.exponentialRampToValueAtTime(140, now + 0.18);
                gain.gain.setValueAtTime(0.14, now);
                gain.gain.exponentialRampToValueAtTime(0.001, now + 0.18);
                osc.connect(gain);
                gain.connect(this.ctx.destination);
                osc.start(now);
                osc.stop(now + 0.18);

                // Deflective shield/metal ring
                const osc2 = this.ctx.createOscillator();
                const gain2 = this.ctx.createGain();
                osc2.type = 'sine';
                osc2.frequency.setValueAtTime(1860, now);
                osc2.frequency.exponentialRampToValueAtTime(620, now + 0.35);
                gain2.gain.setValueAtTime(0.08, now);
                gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.35);
                osc2.connect(gain2);
                gain2.connect(this.ctx.destination);
                osc2.start(now);
                osc2.stop(now + 0.35);
            } catch (_) {}
        },
        playTribute() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Cascading gold coins arpeggio
                [987.77, 1174.66, 1318.51, 1567.98, 1975.53].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(freq, now + i * 0.05);
                    gain.gain.setValueAtTime(0.09, now + i * 0.05);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.05 + 0.22);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.05);
                    osc.stop(now + i * 0.05 + 0.22);
                });
            } catch (_) {}
        },
        playSiren() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Two-tone tactical alarm (StarCraft base under attack style)
                [620, 840, 620, 840].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sawtooth';
                    osc.frequency.setValueAtTime(freq, now + i * 0.1);
                    gain.gain.setValueAtTime(0.07, now + i * 0.1);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.1 + 0.09);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.1);
                    osc.stop(now + i * 0.1 + 0.09);
                });
            } catch (_) {}
        },
        playServoLock() {
            if (!this.enabled) return;
            this.init();
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                // Hydraulic servo clamp & posture lock
                const osc = this.ctx.createOscillator();
                const gain = this.ctx.createGain();
                osc.type = 'sawtooth';
                osc.frequency.setValueAtTime(120, now);
                osc.frequency.exponentialRampToValueAtTime(50, now + 0.16);
                gain.gain.setValueAtTime(0.12, now);
                gain.gain.exponentialRampToValueAtTime(0.001, now + 0.16);
                osc.connect(gain);
                gain.connect(this.ctx.destination);
                osc.start(now);
                osc.stop(now + 0.16);
            } catch (_) {}
        },
    },

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
        this.minimapCanvas = el('cg-minimap');
        if (this.minimapCanvas) this.minimapCtx = this.minimapCanvas.getContext('2d');
        this.avatarCanvas = el('entity-avatar-canvas');
        if (this.avatarCanvas) this.avatarCtx = this.avatarCanvas.getContext('2d');

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

        this.initEnvWeather();
        this.bindEvents();
    },

    initEnvWeather(worldName = '') {
        const wn = (worldName || '').toLowerCase();
        let type = 'cyber';
        if (wn.includes('eco')) type = 'ecosystem';
        else if (wn.includes('coast')) type = 'coastal';
        else if (wn.includes('supply') || wn.includes('freight')) type = 'industrial';
        else if (wn.includes('stress')) type = 'crisis';
        else if (wn.includes('city')) type = 'urban';
        this.envWeatherType = type;

        const count = 48;
        this.envWeather = Array.from({ length: count }, (_, i) => ({
            id: i,
            x: (Math.random() - 0.5) * 1400,
            y: (Math.random() - 0.5) * 900,
            r: type === 'ecosystem' ? Math.random() * 2.2 + 1.2 : Math.random() * 1.8 + 0.8,
            speedX: type === 'coastal' ? 0.9 + Math.random() * 0.8 : (Math.random() - 0.5) * 0.35,
            speedY: type === 'ecosystem' ? -0.25 - Math.random() * 0.3 : (Math.random() - 0.5) * 0.35,
            alpha: Math.random() * 0.45 + 0.15,
            phase: Math.random() * Math.PI * 2,
            color: type === 'ecosystem' ? (Math.random() > 0.35 ? '#a3e635' : '#facc15')
                 : type === 'coastal' ? (Math.random() > 0.5 ? '#38bdf8' : '#e0f2fe')
                 : type === 'crisis' ? (Math.random() > 0.5 ? '#f43f5e' : '#fb923c')
                 : (Math.random() > 0.5 ? '#67a9ff' : '#42d3ea'),
        }));
    },

    syncAudioBtn(active) {
        const btn = el('cg-btn-audio');
        const iconOn = el('cg-icon-audio-on');
        const iconOff = el('cg-icon-audio-off');
        if (btn) btn.classList.toggle('audio-active', active);
        if (iconOn) iconOn.classList.toggle('hidden', !active);
        if (iconOff) iconOff.classList.toggle('hidden', active);
    },

    toggleFullscreen() {
        const wrap = el('play-network-container');
        if (!wrap) return;
        if (document.fullscreenElement) {
            document.exitFullscreen().catch(() => {});
            wrap.classList.remove('fullscreen-active');
        } else if (wrap.requestFullscreen) {
            wrap.requestFullscreen().catch(() => {
                wrap.classList.toggle('fullscreen-active');
            });
        } else {
            wrap.classList.toggle('fullscreen-active');
        }
        setTimeout(() => this.fitView(false), 120);
    },

    bindKeyboard() {
        window.addEventListener('keydown', (e) => {
            if (this.viewMode === 'schematic' || !this.active) return;
            const tag = document.activeElement?.tagName?.toLowerCase();
            if (tag === 'input' || tag === 'textarea' || tag === 'select') return;

            const key = e.key.toLowerCase();
            if (key === 'escape' && this.perspectiveMode !== 'strategic') {
                e.preventDefault();
                this.setViewMode('cg');
                return;
            }
            if (this.perspectiveMode !== 'strategic' && ['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'].includes(key)) {
                e.preventDefault();
                this.pressedKeys.add(key);
                return;
            }
            if (key === 'w' || e.key === 'ArrowUp') {
                e.preventDefault();
                this.camera.targetY += 45;
                this.camera.hasInteracted = true;
            } else if (key === 's' || e.key === 'ArrowDown') {
                e.preventDefault();
                this.camera.targetY -= 45;
                this.camera.hasInteracted = true;
            } else if (key === 'a' || e.key === 'ArrowLeft') {
                e.preventDefault();
                this.camera.targetX += 45;
                this.camera.hasInteracted = true;
            } else if (key === 'd' || e.key === 'ArrowRight') {
                e.preventDefault();
                this.camera.targetX -= 45;
                this.camera.hasInteracted = true;
            } else if (key === '+' || key === '=') {
                e.preventDefault();
                this.zoomBy(1.22);
            } else if (key === '-' || key === '_') {
                e.preventDefault();
                this.zoomBy(0.82);
            } else if (key === 'f' || e.key === 'Home') {
                e.preventDefault();
                this.fitView(true);
            } else if (e.code === 'Space') {
                e.preventDefault();
                togglePlay();
            } else if (key === 'm') {
                e.preventDefault();
                this.setViewMode(this.viewMode === 'cg' ? 'schematic' : 'cg');
            } else if (key === 'escape' && this.placementBuilding) {
                e.preventDefault();
                this.setPlacementBuilding(null);
                return;
            } else if (['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', 'm'].includes(key)) {
                if (this.worldLens === 'city') {
                    e.preventDefault();
                    const bMap = {
                        '1': 'courtyard-homes',
                        '2': 'arcology-spine',
                        '3': 'solar-canopy',
                        '4': 'fusion-reactor',
                        '5': 'water-garden',
                        '6': 'vertical-farm',
                        '7': 'maker-cooperative',
                        '8': 'nanotech-foundry',
                        '9': 'hyperloop-exchange',
                        '0': 'civic-lab',
                        '-': 'quantum-observatory',
                        '=': 'story-forum',
                        'm': 'grand-amphitheater',
                    };
                    if (bMap[key]) this.setPlacementBuilding(bMap[key]);
                } else {
                    const speeds = { '1': 1, '2': 2, '3': 5, '4': 10, '5': 20 };
                    const speedSelect = el('play-speed');
                    if (speedSelect && speeds[key]) {
                        speedSelect.value = String(speeds[key]);
                        setPlaySpeed(speeds[key]);
                    }
                }
            }
        });
        window.addEventListener('keyup', e => this.pressedKeys.delete(e.key.toLowerCase()));
    },

    bindControls() {
        this.init();
        el('btn-view-cg')?.addEventListener('click', () => this.setViewMode('cg'));
        el('btn-view-schematic')?.addEventListener('click', () => this.setViewMode('schematic'));
        el('btn-view-city')?.addEventListener('click', () => this.setViewMode('city'));
        el('btn-view-realm')?.addEventListener('click', () => this.setViewMode('realm'));
        el('btn-view-third')?.addEventListener('click', () => this.setViewMode('third'));
        el('btn-view-first')?.addEventListener('click', () => this.setViewMode('first'));
        el('cg-btn-zoom-in')?.addEventListener('click', () => this.zoomBy(1.28));
        el('cg-btn-zoom-out')?.addEventListener('click', () => this.zoomBy(0.78));
        el('cg-btn-reset-cam')?.addEventListener('click', () => this.fitView(true));
        el('cg-btn-fx-toggle')?.addEventListener('click', () => this.toggleVfx());
        el('cg-btn-audio')?.addEventListener('click', () => {
            const active = this.audio.toggle();
            this.syncAudioBtn(active);
            if (active) this.audio.playBlip(750, 0.08);
            showToast('Tactical Audio', active ? 'Procedural sound FX active.' : 'Procedural sound FX muted.');
        });
        el('cg-btn-fullscreen')?.addEventListener('click', () => this.toggleFullscreen());
        el('immersion-pause')?.addEventListener('click', () => togglePlay());
        el('immersion-checkpoint')?.addEventListener('click', () => openSaveManager(true));
        el('immersion-intervene')?.addEventListener('click', () => {
            const target = el('decision-console') || el('play-entity-select');
            target?.scrollIntoView({ behavior: 'smooth', block: 'center' });
            el('capacity-slider')?.focus();
            setPlayRunning(false);
            showToast('Intervention window', 'Simulation paused. Adjust capacity, construction, research, governance, or covert operations.');
        });
        this.syncAudioBtn(this.audio.enabled);
        this.bindKeyboard();
    },

    bindEvents() {
        const c = this.canvas;
        if (!c) return;

        // Minimap drag pan
        if (this.minimapCanvas) {
            let isMinimapDragging = false;
            const panFromMinimap = (e) => {
                const rect = this.minimapCanvas.getBoundingClientRect();
                const mx = e.clientX - rect.left;
                const my = e.clientY - rect.top;
                this.panToMinimapCoord(mx, my);
            };
            this.minimapCanvas.addEventListener('pointerdown', e => {
                if (e.button !== 0) return;
                isMinimapDragging = true;
                this.minimapCanvas.setPointerCapture(e.pointerId);
                panFromMinimap(e);
            });
            this.minimapCanvas.addEventListener('pointermove', e => {
                if (isMinimapDragging) panFromMinimap(e);
            });
            const stopMinimapDrag = e => {
                if (isMinimapDragging) {
                    isMinimapDragging = false;
                    try { this.minimapCanvas.releasePointerCapture(e.pointerId); } catch (_) {}
                }
            };
            this.minimapCanvas.addEventListener('pointerup', stopMinimapDrag);
            this.minimapCanvas.addEventListener('pointercancel', stopMinimapDrag);
        }

        // Pointer / mouse drag
        c.addEventListener('pointerdown', e => {
            if (e.button !== 0) return;
            c.setPointerCapture(e.pointerId);
            this.camera.isDragging = true;
            this.camera.dragStartX = e.clientX;
            this.camera.dragStartY = e.clientY;
            this.camera.camStartX = this.camera.targetX;
            this.camera.camStartY = this.camera.targetY;
            this.camera.headingStart = this.walker.heading;
        });

        window.addEventListener('pointermove', e => {
            if (this.camera.isDragging) {
                const dx = e.clientX - this.camera.dragStartX;
                const dy = e.clientY - this.camera.dragStartY;
                if (this.perspectiveMode === 'first') {
                    this.walker.heading = this.camera.headingStart + dx * 0.006;
                    this.camera.targetZoom = Math.max(2.2, Math.min(4.2, this.camera.targetZoom - dy * 0.004));
                } else {
                    this.camera.targetX = this.camera.camStartX + dx;
                    this.camera.targetY = this.camera.camStartY + dy;
                }
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

            if (this.worldLens === 'city') {
                const tile = this.fromIso(px, py);
                if (tile.gx >= 0 && tile.gx < 10 && tile.gy >= 0 && tile.gy < 10) {
                    const district = this.getDistrictForTile(tile.gx, tile.gy);
                    if (this.placementBuilding) {
                        if (district) {
                            constructCityBuildingDirect(this.placementBuilding, district);
                            this.cityPlotMap = this.cityPlotMap || new Map();
                            this.cityPlotMap.set(`${tile.gx},${tile.gy}`, {
                                id: this.placementBuilding,
                                level: 1,
                                name: prettyName(this.placementBuilding)
                            });
                            this.audio.playConstruction();
                            const iso = this.toIso(tile.gx, tile.gy);
                            this.shockwaves.push({
                                x: iso.x, y: iso.y, r: 10, maxR: 70,
                                color: '#38ef7d', alpha: 1, width: 3,
                            });
                            if (!e.shiftKey) {
                                this.setPlacementBuilding(null);
                            }
                        } else {
                            showToast('Invalid Plot', 'Cannot construct on roadways or public plaza.', 'error');
                            this.audio.playShortage();
                        }
                    } else {
                        const existing = this.getBuildingAtTile(tile.gx, tile.gy);
                        if (existing) {
                            openBuildingInspector(existing.id, district);
                        } else if (district) {
                            showToast('District Selected', `${prettyName(district)} selected.`);
                            this.audio.playBlip(720, 0.05);
                        }
                    }
                }
                return;
            } else if (this.worldLens === 'realm') {
                const realmHit = this.getRealmEntityAt(px, py);
                if (realmHit) {
                    openWarRoomModal();
                    this.audio.playWarHorn();
                }
                return;
            }

            const hit = this.getNodeAt(px, py);
            if (hit) {
                this.setSelectedEntity(hit.name);
                this.audio.playBlip(720, 0.06);
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
        this.viewMode = mode === 'schematic' ? 'schematic' : 'cg';
        this.perspectiveMode = mode === 'first' || mode === 'third' ? mode : 'strategic';
        if (mode === 'city' || mode === 'realm' || mode === 'cg') this.worldLens = mode;
        const btnCg = el('btn-view-cg');
        const btnSchem = el('btn-view-schematic');
        const btnFirst = el('btn-view-first');
        const btnThird = el('btn-view-third');
        const btnCity = el('btn-view-city');
        const btnRealm = el('btn-view-realm');
        const canvas = el('play-cg-canvas');
        const svg = el('play-network');
        const toolbar = el('cg-toolbar');
        const hud = el('cg-hud-card');

        if (mode !== 'schematic') {
            btnCg?.classList.toggle('active', this.perspectiveMode === 'strategic' && this.worldLens === 'cg');
            btnCity?.classList.toggle('active', this.perspectiveMode === 'strategic' && this.worldLens === 'city');
            btnRealm?.classList.toggle('active', this.perspectiveMode === 'strategic' && this.worldLens === 'realm');
            btnSchem?.classList.remove('active');
            btnFirst?.classList.toggle('active', this.perspectiveMode === 'first');
            btnThird?.classList.toggle('active', this.perspectiveMode === 'third');
            canvas?.classList.remove('hidden');
            svg?.classList.add('hidden');
            toolbar?.classList.remove('hidden');
            el('immersion-hud')?.classList.toggle('hidden', this.perspectiveMode === 'strategic');
            el('city-build-dock')?.classList.toggle('hidden', this.worldLens !== 'city');
            el('play-network-container')?.setAttribute('data-perspective', this.perspectiveMode);
            if (this.perspectiveMode !== 'strategic') this.enterImmersiveMode();
            this.start();
        } else {
            btnCg?.classList.remove('active');
            btnSchem?.classList.add('active');
            btnFirst?.classList.remove('active');
            btnThird?.classList.remove('active');
            btnCity?.classList.remove('active');
            btnRealm?.classList.remove('active');
            canvas?.classList.add('hidden');
            svg?.classList.remove('hidden');
            toolbar?.classList.add('hidden');
            hud?.classList.add('hidden');
            el('immersion-hud')?.classList.add('hidden');
            el('city-build-dock')?.classList.add('hidden');
            el('play-network-container')?.removeAttribute('data-perspective');
            this.stop();
        }
    },

    enterImmersiveMode() {
        const focus = this.nodes.get(this.selectedNodeName) || this.nodes.values().next().value;
        if (focus) {
            this.walker.x = focus.x;
            this.walker.y = focus.y + focus.radius * 2.2;
            this.selectedNodeName = focus.name;
        }
        this.camera.hasInteracted = false;
        this.camera.targetZoom = this.perspectiveMode === 'first' ? 3.05 : 1.65;
        el('immersion-mode').textContent = this.perspectiveMode === 'first' ? 'FIRST-PERSON CITY LINK' : 'THIRD-PERSON FOLLOW';
        el('immersion-help').textContent = this.perspectiveMode === 'first'
            ? 'WASD move · drag look · wheel field of view · Esc tactical'
            : 'WASD orbit · click entity to follow · wheel distance · Esc tactical';
        this.updateImmersionLabels();
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

        if (data.world && this.lastWorld !== data.world) {
            this.lastWorld = data.world;
            this.initEnvWeather(data.world);
        }

        if (data.city && !this.hasChosenLens) {
            this.hasChosenLens = true;
            this.worldLens = 'city';
            this.setViewMode('city');
        }

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

        if (!this.selectedNodeName && entities.length > 0) {
            this.setSelectedEntity(entities[0].name);
        } else if (this.selectedNodeName) {
            this.updateAvatarHero(this.selectedNodeName);
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
            // Tiered flow layout (left to right) with generous vertical spacing
            const sortedRanks = [...rankGroups.keys()].sort((a, b) => a - b);
            const totalCols = sortedRanks.length;
            const widthSpan = count > 10 ? 680 : 560;
            const xStep = totalCols > 1 ? widthSpan / (totalCols - 1) : 0;
            const startX = -widthSpan / 2;

            sortedRanks.forEach((rank, colIdx) => {
                const colNodes = rankGroups.get(rank);
                const colCount = colNodes.length;
                const heightSpan = Math.max(160, (colCount - 1) * 165);
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
            const rx = count > 14 ? 330 : 280;
            const ry = count > 14 ? 180 : 150;
            entities.forEach((e, idx) => {
                const angle = (idx / count) * Math.PI * 2 - Math.PI / 2;
                positions.set(e.name, {
                    x: Math.round(rx * Math.cos(angle)),
                    y: Math.round(ry * Math.sin(angle)),
                });
            });
        }

        // Repulsion relaxation pass to guarantee collision-free placement
        const posArray = [...positions.entries()];
        for (let iter = 0; iter < 12; iter++) {
            for (let i = 0; i < posArray.length; i++) {
                for (let j = i + 1; j < posArray.length; j++) {
                    const p1 = posArray[i][1];
                    const p2 = posArray[j][1];
                    const dx = p2.x - p1.x;
                    const dy = p2.y - p1.y;
                    const dist = Math.hypot(dx, dy) || 1;
                    const minDist = 150;
                    if (dist < minDist) {
                        const overlap = (minDist - dist) / 2;
                        const nx = dx / dist;
                        const ny = dy / dist;
                        p1.x -= nx * overlap;
                        p1.y -= ny * overlap;
                        p2.x += nx * overlap;
                        p2.y += ny * overlap;
                    }
                }
            }
        }

        return positions;
    },

    onEvents(events) {
        if (!events || !events.length) return;

        events.forEach(event => {
            const reactive = {
                shortage: ['rgb(255,111,124)', .2],
                governance: ['rgb(66,211,234)', .13],
                geopolitics: ['rgb(255,111,124)', .16],
                intrigue: ['rgb(241,185,107)', .15],
                construction: ['rgb(70,210,154)', .12],
                research: ['rgb(213,120,239)', .13],
            }[event.type];
            if (reactive) {
                this.immersionPulse = { color: reactive[0], alpha: reactive[1], label: event.summary || '' };
                if (el('immersion-signal')) el('immersion-signal').textContent = event.summary || `${prettyName(event.type)} event`;
            }
        });

        // Group events by entity to aggregate identical concurrent items
        const byEntity = new Map();
        events.forEach(ev => {
            if (!byEntity.has(ev.entity)) byEntity.set(ev.entity, []);
            byEntity.get(ev.entity).push(ev);
        });

        let hadShortage = false;
        let hadCapacityChange = false;

        byEntity.forEach((evList, entityName) => {
            const node = this.nodes.get(entityName);
            if (!node) return;

            // Aggregate identical production events: resource -> total amount & count
            const prodAgg = new Map();
            const otherEvents = [];

            evList.forEach(ev => {
                if (ev.type === 'production') {
                    const key = ev.resource || 'unknown';
                    const cur = prodAgg.get(key) || { amount: 0, count: 0, resource: ev.resource };
                    cur.amount += (ev.amount || 0);
                    cur.count += 1;
                    prodAgg.set(key, cur);
                } else {
                    otherEvents.push(ev);
                }
            });

            // Trigger pulses
            if (prodAgg.size > 0) {
                node.pulse = 1.0;
                if (this.vfxEnabled) {
                    this.shockwaves.push({
                        x: node.x, y: node.y, r: node.radius, maxR: node.radius + 38,
                        color: '#46d29a', alpha: 0.9, width: 2.0,
                    });
                }
            }

            // Collect all unique floatie descriptors for this entity
            const floatieItems = [];
            prodAgg.forEach((agg, res) => {
                const countSuffix = agg.count > 1 ? ` (x${agg.count})` : '';
                floatieItems.push({
                    text: `+${compactNumber(agg.amount)} ${prettyName(res)}${countSuffix}`,
                    color: getResourceNeonColor(res) || '#46d29a',
                    type: 'prod',
                });
            });

            otherEvents.forEach(ev => {
                if (ev.type === 'shortage') {
                    hadShortage = true;
                    node.warningPulse = 1.0;
                    if (this.vfxEnabled) {
                        this.shockwaves.push({
                            x: node.x, y: node.y, r: node.radius, maxR: node.radius + 50,
                            color: '#ff6f7c', alpha: 1.0, width: 2.6,
                        });
                    }
                    floatieItems.push({
                        text: `⚠ SHORTAGE: ${prettyName(ev.resource)}`,
                        color: '#ff6f7c',
                        type: 'shortage',
                    });
                } else if (ev.type === 'capacity_change') {
                    hadCapacityChange = true;
                    if (this.vfxEnabled) {
                        this.shockwaves.push({
                            x: node.x, y: node.y, r: node.radius, maxR: node.radius + 32,
                            color: '#f1b96b', alpha: 0.85, width: 2.0,
                        });
                    }
                    floatieItems.push({
                        text: `⚡ ${Math.round(ev.value * 100)}%`,
                        color: '#f1b96b',
                        type: 'capacity',
                    });
                }
            });

            if (this.vfxEnabled && floatieItems.length > 0) {
                const total = floatieItems.length;
                floatieItems.forEach((item, idx) => {
                    // Stagger horizontally and vertically so each item is distinctly readable
                    const xOffset = total > 1 ? (idx - (total - 1) / 2) * 36 : 0;
                    const yOffset = idx * 15;
                    this.floaties.push({
                        text: item.text,
                        x: node.x + xOffset,
                        y: node.y - node.radius - 12 - yOffset,
                        vy: -0.9 - Math.random() * 0.3,
                        color: item.color,
                        type: item.type,
                        alpha: 1,
                        life: item.type === 'shortage' ? 85 : 65,
                        maxLife: item.type === 'shortage' ? 85 : 65,
                    });
                });
            }
        });

        // Procedural Audio SFX
        if (hadShortage) {
            this.audio.playShortage();
        } else if (hadCapacityChange) {
            this.audio.playOverdrive();
        }
    },

    setSelectedEntity(name) {
        this.selectedNodeName = name;
        this.updateAvatarHero(name);
        this.updateImmersionLabels();
    },

    updateImmersionLabels() {
        const focus = this.nodes.get(this.selectedNodeName);
        const label = focus ? prettyName(focus.name) : 'Free camera';
        if (el('immersion-focus')) el('immersion-focus').textContent = label;
    },

    updateImmersiveCamera(now) {
        if (this.perspectiveMode === 'strategic') return;
        const focus = this.nodes.get(this.selectedNodeName) || this.nodes.values().next().value;
        const key = value => this.pressedKeys.has(value);
        if (this.perspectiveMode === 'first') {
            const forward = (key('w') || key('arrowup') ? 1 : 0) - (key('s') || key('arrowdown') ? 1 : 0);
            const strafe = (key('d') ? 1 : 0) - (key('a') ? 1 : 0);
            if (key('arrowleft')) this.walker.heading -= 0.025;
            if (key('arrowright')) this.walker.heading += 0.025;
            const speed = key('shift') ? 3.2 : 1.65;
            this.walker.x += (Math.sin(this.walker.heading) * forward + Math.cos(this.walker.heading) * strafe) * speed;
            this.walker.y += (-Math.cos(this.walker.heading) * forward + Math.sin(this.walker.heading) * strafe) * speed;
            this.walker.bob += Math.abs(forward || strafe) * 0.13;
            const bob = (forward || strafe) ? Math.sin(this.walker.bob) * 3 : 0;
            this.camera.targetX = -this.walker.x * this.camera.targetZoom;
            this.camera.targetY = -this.walker.y * this.camera.targetZoom + (this.canvas?.clientHeight || 420) * 0.17 + bob;
        } else if (focus) {
            if (key('a') || key('arrowleft')) this.walker.heading -= 0.022;
            if (key('d') || key('arrowright')) this.walker.heading += 0.022;
            const distanceShift = (key('s') || key('arrowdown') ? 1 : 0) - (key('w') || key('arrowup') ? 1 : 0);
            this.camera.targetZoom = Math.max(0.9, Math.min(2.5, this.camera.targetZoom - distanceShift * 0.012));
            const orbit = 48 / this.camera.targetZoom;
            this.walker.x = focus.x + Math.sin(this.walker.heading) * orbit;
            this.walker.y = focus.y + Math.cos(this.walker.heading) * orbit;
            this.camera.targetX = -this.walker.x * this.camera.targetZoom;
            this.camera.targetY = -this.walker.y * this.camera.targetZoom + (this.canvas?.clientHeight || 420) * 0.08;
        }
        if (el('immersion-signal')) {
            const tick = state.play.data?.currentTick || 0;
            el('immersion-signal').textContent = state.play.running ? `Live simulation · tick ${formatNumber(tick)}` : `Time frozen · tick ${formatNumber(tick)}`;
        }
    },

    updateAvatarHero(name) {
        const node = this.nodes.get(name) || (this.nodes.size > 0 ? this.nodes.values().next().value : null);
        if (!node) return;
        const nameEl = el('avatar-entity-name');
        if (nameEl) nameEl.textContent = prettyName(node.name);
        const archEl = el('avatar-entity-archetype');
        if (archEl) {
            archEl.textContent = node.theme.label;
            archEl.style.color = node.theme.color;
        }
        const regEl = el('avatar-entity-region');
        if (regEl) regEl.textContent = `Sector: ${prettyName(node.region)} · ${node.archetype.toUpperCase()}`;
        const pipEl = el('avatar-status-pip');
        if (pipEl) {
            pipEl.className = 'avatar-status-pip';
            if (node.capacity > 1.0) {
                pipEl.classList.add('overdrive');
            } else if (node.warningPulse > 0.05 || node.capacity === 0) {
                pipEl.classList.add('warning');
            } else {
                pipEl.classList.add('active');
            }
        }
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
        if (this.worldLens === 'city') {
            const tile = this.fromIso(px, py);
            if (tile.gx >= 0 && tile.gx < 10 && tile.gy >= 0 && tile.gy < 10) {
                this.hoveredTile = tile;
                this.hoveredDistrict = this.getDistrictForTile(tile.gx, tile.gy);
                this.canvas.style.cursor = this.placementBuilding ? 'crosshair' : 'pointer';
            } else {
                this.hoveredTile = null;
                this.hoveredDistrict = null;
                this.canvas.style.cursor = this.camera.isDragging ? 'grabbing' : 'grab';
            }
            if (this.hud) this.hud.classList.add('hidden');
            return;
        } else if (this.worldLens === 'realm') {
            const realmHit = this.getRealmEntityAt(px, py);
            this.hoveredRealmEntity = realmHit;
            this.canvas.style.cursor = realmHit ? 'pointer' : (this.camera.isDragging ? 'grabbing' : 'grab');
            if (this.hud) this.hud.classList.add('hidden');
            return;
        }

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

    initCityLife() {
        if (this.cityLifeInit) return;
        this.cityLifeInit = true;
        const professions = [
            { role: 'Farmer', emoji: '🌾', color: '#84cc16' },
            { role: 'Engineer', emoji: '⚙️', color: '#f59e0b' },
            { role: 'Scientist', emoji: '🧪', color: '#38bdf8' },
            { role: 'Citizen', emoji: '😊', color: '#a855f7' },
            { role: 'Merchant', emoji: '💰', color: '#fbbf24' },
            { role: 'Guard', emoji: '🛡️', color: '#ef4444' }
        ];
        this.cityCitizens = Array.from({ length: 14 }, (_, i) => {
            const prof = professions[i % professions.length];
            const onHWay = i % 2 === 0;
            return {
                gx: onHWay ? 1 + (i % 8) : (i % 4 < 2 ? 4 : 5),
                gy: onHWay ? (i % 4 < 2 ? 4 : 5) : 1 + (i % 8),
                dir: i % 3 === 0 ? -1 : 1,
                axis: onHWay ? 'x' : 'y',
                speed: 0.012 + (i % 3) * 0.005,
                bob: i * 0.9,
                role: prof.role,
                emoji: prof.emoji,
                color: prof.color,
                bubbleTimer: Math.random() * 200,
                bubbleEmoji: prof.emoji
            };
        });

        this.chimneySmoke = Array.from({ length: 24 }, (_, i) => ({
            gx: 2, gy: 7,
            offsetY: -(i * 2.5),
            offsetX: 0,
            r: 2 + Math.random() * 3.5,
            alpha: Math.random() * 0.7,
            vx: (Math.random() - 0.3) * 0.25,
            vy: -0.45 - Math.random() * 0.3
        }));
    },

    toIso(gx, gy) {
        const tileW = 76;
        const tileH = 38;
        return {
            x: (gx - gy) * (tileW / 2),
            y: (gx + gy) * (tileH / 2)
        };
    },

    fromIso(px, py) {
        const w = this.canvas.clientWidth;
        const h = this.canvas.clientHeight;
        const cx = w / 2 + this.camera.x;
        const cy = h / 2 + this.camera.y - 120;
        const wx = (px - cx) / this.camera.zoom;
        const wy = (py - cy) / this.camera.zoom;
        const tileW = 76;
        const tileH = 38;
        const gx = Math.round((wx / (tileW / 2) + wy / (tileH / 2)) / 2);
        const gy = Math.round((wy / (tileH / 2) - wx / (tileW / 2)) / 2);
        return { gx, gy };
    },

    getDistrictForTile(gx, gy) {
        if (gx < 0 || gx > 9 || gy < 0 || gy > 9) return null;
        if (gx === 4 || gx === 5 || gy === 4 || gy === 5) return null; // Roads & Plaza
        if (gx === 0 || gx === 9 || gy === 0 || gy === 9) return null; // Perimeter

        if (gx >= 1 && gx <= 3 && gy >= 1 && gy <= 3) return 'civic-core';
        if (gx >= 6 && gx <= 8 && gy >= 1 && gy <= 3) return 'river-ward';
        if (gx >= 1 && gx <= 3 && gy >= 6 && gy <= 8) return 'old-grid';
        if (gx >= 6 && gx <= 8 && gy >= 6 && gy <= 8) return 'sun-belt';
        return null;
    },

    getBuildingAtTile(gx, gy) {
        if (!this.cityPlotMap) this.cityPlotMap = new Map();
        const key = `${gx},${gy}`;
        if (this.cityPlotMap.has(key)) {
            return this.cityPlotMap.get(key);
        }

        const city = state.play?.data?.city;
        if (!city || !city.buildings) return null;

        const district = this.getDistrictForTile(gx, gy);
        if (!district) return null;

        // Structured slot assignment for each district quadrant
        const districtPlots = {
            'civic-core': [[2,2], [1,2], [2,1], [3,2], [2,3], [1,1], [3,1], [1,3], [3,3]],
            'river-ward': [[7,2], [8,2], [7,1], [6,2], [7,3], [6,1], [8,1], [6,3], [8,3]],
            'old-grid':   [[2,7], [2,6], [1,7], [3,7], [2,8], [1,6], [3,6], [1,8], [3,8]],
            'sun-belt':   [[7,7], [6,7], [7,6], [8,7], [7,8], [6,6], [8,6], [6,8], [8,8]],
        };

        const plots = districtPlots[district] || [];
        const plotIndex = plots.findIndex(([px, py]) => px === gx && py === gy);
        if (plotIndex === -1) return null;

        // Collect all placed buildings in this district
        const buildingsInDistrict = [];
        city.buildings.forEach(b => {
            if (b.count > 0 && (!b.allowed_districts || b.allowed_districts.includes(district))) {
                for (let i = 0; i < b.count; i++) {
                    buildingsInDistrict.push({ id: b.id, level: b.level || 1, name: b.name, category: b.category });
                }
            }
        });

        if (plotIndex < buildingsInDistrict.length) {
            return buildingsInDistrict[plotIndex];
        }

        // Fallback initial starter buildings if city is just founded
        if (district === 'civic-core' && gx === 2 && gy === 2) return { id: 'civic-lab', level: 1, name: 'Civic Lab', category: 'knowledge' };
        if (district === 'civic-core' && gx === 1 && gy === 2) return { id: 'courtyard-homes', level: 1, name: 'Courtyard Homes', category: 'residential' };
        if (district === 'river-ward' && gx === 7 && gy === 2) return { id: 'vertical-farm', level: 1, name: 'Vertical Farm', category: 'food' };
        if (district === 'river-ward' && gx === 8 && gy === 2) return { id: 'water-garden', level: 1, name: 'Living Water Garden', category: 'water' };
        if (district === 'old-grid' && gx === 2 && gy === 7) return { id: 'maker-cooperative', level: 1, name: 'Maker Cooperative', category: 'industry' };
        if (district === 'sun-belt' && gx === 7 && gy === 7) return { id: 'solar-canopy', level: 1, name: 'Solar Canopy', category: 'energy' };

        return null;
    },

    setPlacementBuilding(buildingId) {
        this.placementBuilding = buildingId;
        const status = el('dock-placement-status');
        const nameEl = el('placement-building-name');
        const dock = el('city-build-dock');

        if (dock) {
            dock.querySelectorAll('.dock-build-btn').forEach(btn => {
                btn.classList.toggle('active-placement', btn.dataset.building === buildingId);
            });
        }

        if (buildingId) {
            if (status) status.classList.remove('hidden');
            if (nameEl) nameEl.textContent = `Placing: ${prettyName(buildingId)}`;
            this.canvas.style.cursor = 'crosshair';
            this.audio?.playBlip?.(600, 0.08);
        } else {
            if (status) status.classList.add('hidden');
            this.canvas.style.cursor = 'grab';
        }
    },

    getRealmEntityAt(px, py) {
        const w = this.canvas.clientWidth;
        const h = this.canvas.clientHeight;
        const cx = w / 2 + this.camera.x;
        const cy = h / 2 + this.camera.y;
        const wx = (px - cx) / this.camera.zoom;
        const wy = (py - cy) / this.camera.zoom;

        const realms = [
            { id: 'sanctuary-haven', name: 'Sanctuary Haven', x: 0, y: 0, r: 52 },
            { id: 'barony-oakhaven', name: 'Barony of Oakhaven', x: 0, y: -190, r: 44 },
            { id: 'riverside-vassal', name: 'Riverside Protectorate', x: -260, y: -30, r: 44 },
            { id: 'iron-mandate', name: 'Iron Mandate Hegemony', x: 260, y: -30, r: 44 },
            { id: 'mercantile-league', name: 'Free Mercantile League', x: 120, y: 190, r: 44 },
            { id: 'dust-canyon-raiders', name: 'Dust Canyon Raiders', x: -180, y: 170, r: 44 },
        ];

        for (const r of realms) {
            if (Math.hypot(wx - r.x, wy - r.y) <= r.r) return r;
        }
        return null;
    },

    renderCityView(ctx, w, h, now) {
        this.initCityLife();
        const city = state.play?.data?.city;
        const eco = city?.ecology;
        const season = (eco?.season || 'Spring').toLowerCase();
        const weather = (eco?.weather || 'Clear').toLowerCase();
        const tick = state.play?.data?.tick || 0;

        const diurnalHour = (tick * 0.25) % 24;
        this.currentDiurnalHour = diurnalHour;
        const isNight = diurnalHour >= 19.5 || diurnalHour < 5.5;
        const isDusk = diurnalHour >= 17 && diurnalHour < 19.5;
        const isDawn = diurnalHour >= 5.5 && diurnalHour < 8;

        const dIndex = city?.geopolitics?.droughtIndex || 0;
        const isDrought = dIndex > 35 || weather === 'drought';
        const isWinter = season === 'winter' || weather === 'snow';
        const isAutumn = season === 'autumn';
        const isSpring = season === 'spring';
        const isFortified = city?.geopolitics?.defensePosture === 'fortified';
        const raidThreat = city?.geopolitics?.borderThreat || 0;
        const tileW = 76;
        const tileH = 38;

        ctx.save();
        ctx.translate(0, -120);

        const tiles = [];
        for (let gx = 0; gx < 10; gx++) {
            for (let gy = 0; gy < 10; gy++) {
                tiles.push({ gx, gy, depth: (gx + gy) * 100 + (gx - gy) });
            }
        }
        tiles.sort((a, b) => a.depth - b.depth);

        tiles.forEach(({ gx, gy }) => {
            const { x: ix, y: iy } = this.toIso(gx, gy);
            const isRoad = (gx === 4 || gx === 5 || gy === 4 || gy === 5);
            const isPlaza = (gx === 4 || gx === 5) && (gy === 4 || gy === 5);
            const isPerimeter = (gx === 0 || gx === 9 || gy === 0 || gy === 9);
            const district = this.getDistrictForTile(gx, gy);

            // Draw isometric depth slab
            ctx.fillStyle = '#0f172a';
            ctx.beginPath();
            ctx.moveTo(ix - tileW / 2, iy + tileH / 2);
            ctx.lineTo(ix, iy + tileH);
            ctx.lineTo(ix + tileW / 2, iy + tileH / 2);
            ctx.lineTo(ix + tileW / 2, iy + tileH / 2 + 10);
            ctx.lineTo(ix, iy + tileH + 10);
            ctx.lineTo(ix - tileW / 2, iy + tileH / 2 + 10);
            ctx.closePath();
            ctx.fill();

            // Tile top
            ctx.beginPath();
            ctx.moveTo(ix, iy);
            ctx.lineTo(ix + tileW / 2, iy + tileH / 2);
            ctx.lineTo(ix, iy + tileH);
            ctx.lineTo(ix - tileW / 2, iy + tileH / 2);
            ctx.closePath();

            if (isPlaza) {
                const grad = ctx.createLinearGradient(ix - tileW / 2, iy, ix + tileW / 2, iy + tileH);
                grad.addColorStop(0, '#475569');
                grad.addColorStop(1, '#334155');
                ctx.fillStyle = grad;
                ctx.fill();
                ctx.strokeStyle = 'rgba(234, 179, 8, 0.4)';
                ctx.lineWidth = 1;
                ctx.stroke();
            } else if (isRoad) {
                ctx.fillStyle = isNight ? '#090e1a' : '#1e293b';
                ctx.fill();
                ctx.strokeStyle = '#334155';
                ctx.lineWidth = 1;
                ctx.stroke();

                ctx.save();
                ctx.strokeStyle = isNight ? 'rgba(253, 224, 71, 0.65)' : 'rgba(253, 224, 71, 0.45)';
                ctx.setLineDash([4, 4]);
                ctx.beginPath();
                if (gx === 4 || gx === 5) {
                    ctx.moveTo(ix, iy + 6);
                    ctx.lineTo(ix, iy + tileH - 6);
                } else {
                    ctx.moveTo(ix - 18, iy + tileH / 2);
                    ctx.lineTo(ix + 18, iy + tileH / 2);
                }
                ctx.stroke();
                ctx.restore();

                // Night streetlamps
                if (isNight || isDusk) {
                    this.drawRoadStreetlamps(ctx, gx, gy, ix, iy, now);
                }
            } else if (isPerimeter) {
                ctx.fillStyle = isWinter ? '#334155' : (isDrought ? '#3d2b1f' : '#1e293b');
                ctx.fill();
                ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
                ctx.stroke();
            } else {
                const grad = ctx.createLinearGradient(ix, iy, ix, iy + tileH);
                if (isWinter) {
                    grad.addColorStop(0, '#475569');
                    grad.addColorStop(1, '#334155');
                } else if (isAutumn) {
                    grad.addColorStop(0, '#543a1f');
                    grad.addColorStop(1, '#3f2914');
                } else if (isDrought) {
                    grad.addColorStop(0, '#543d2b');
                    grad.addColorStop(1, '#3f2c1d');
                } else if (isSpring) {
                    grad.addColorStop(0, '#1c4d28');
                    grad.addColorStop(1, '#13381c');
                } else {
                    grad.addColorStop(0, '#2d451b');
                    grad.addColorStop(1, '#1f3012');
                }
                ctx.fillStyle = grad;
                ctx.fill();

                // Seasonal flora flecks
                if (isSpring) {
                    ctx.fillStyle = Math.sin(gx * 7 + gy) > 0 ? '#f472b6' : '#fef08a';
                    ctx.fillRect(ix - 8, iy + 14, 2, 2);
                    ctx.fillRect(ix + 6, iy + 22, 2, 2);
                } else if (isWinter) {
                    ctx.fillStyle = 'rgba(255, 255, 255, 0.45)';
                    ctx.fillRect(ix - 10, iy + 10, 4, 1.5);
                    ctx.fillRect(ix + 8, iy + 24, 3, 1.5);
                }

                ctx.strokeStyle = district === 'civic-core' ? 'rgba(66, 211, 234, 0.28)' :
                                  district === 'river-ward' ? 'rgba(56, 189, 248, 0.28)' :
                                  district === 'old-grid' ? 'rgba(245, 158, 11, 0.28)' :
                                  'rgba(234, 179, 8, 0.28)';
                ctx.lineWidth = 1;
                ctx.stroke();
            }

            const isHovered = this.hoveredTile?.gx === gx && this.hoveredTile?.gy === gy;
            if (isHovered) {
                ctx.save();
                ctx.beginPath();
                ctx.moveTo(ix, iy);
                ctx.lineTo(ix + tileW / 2, iy + tileH / 2);
                ctx.lineTo(ix, iy + tileH);
                ctx.lineTo(ix - tileW / 2, iy + tileH / 2);
                ctx.closePath();

                if (this.placementBuilding) {
                    if (district) {
                        ctx.fillStyle = 'rgba(16, 185, 129, 0.3)';
                        ctx.strokeStyle = '#10b981';
                        ctx.lineWidth = 2.5;
                    } else {
                        ctx.fillStyle = 'rgba(239, 68, 68, 0.3)';
                        ctx.strokeStyle = '#ef4444';
                        ctx.lineWidth = 2.5;
                    }
                } else {
                    ctx.fillStyle = 'rgba(245, 158, 11, 0.22)';
                    ctx.strokeStyle = '#f59e0b';
                    ctx.lineWidth = 2;
                }
                ctx.fill();
                ctx.stroke();
                ctx.restore();
            }

            this.drawTileStructure(ctx, gx, gy, ix, iy, now, isHovered && this.placementBuilding);
        });

        this.drawChimneySmoke(ctx, now);
        this.drawCitizens(ctx, now);
        this.drawLogisticsCaravans(ctx, now, eco);
        this.drawSkyDrones(ctx, now);

        if (raidThreat > 25) {
            this.drawRaiders(ctx, now, raidThreat);
        }

        if (isFortified) {
            this.drawShieldDome(ctx, now);
        }

        this.drawAtmosphericWeather(ctx, w, h, now, weather, season);

        // Ambient nocturnal lighting tint
        if (isNight || isDusk) {
            const alpha = isNight ? 0.42 : 0.22;
            ctx.save();
            ctx.fillStyle = `rgba(8, 15, 36, ${alpha})`;
            ctx.fillRect(-w * 2, -h * 2, w * 4, h * 4);
            ctx.restore();
        }

        ctx.restore();
    },

    drawRoadStreetlamps(ctx, gx, gy, ix, iy, now) {
        if (!((gx === 4 && gy === 4) || (gx === 5 && gy === 4) || (gx === 4 && gy === 5) || (gx === 5 && gy === 5) || (gx === 4 && gy === 1) || (gx === 5 && gy === 8) || (gx === 1 && gy === 4) || (gx === 8 && gy === 5))) {
            return;
        }
        ctx.save();
        const lampX = ix + 12;
        const lampY = iy + 10;

        // Lamp post
        ctx.strokeStyle = '#475569';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.moveTo(lampX, lampY);
        ctx.lineTo(lampX, lampY - 18);
        ctx.lineTo(lampX - 4, lampY - 21);
        ctx.stroke();

        // Lamp light bulb
        ctx.fillStyle = '#fef08a';
        ctx.beginPath();
        ctx.arc(lampX - 4, lampY - 21, 2.5, 0, Math.PI * 2);
        ctx.fill();

        // Ambient light pool on ground
        const poolGrad = ctx.createRadialGradient(lampX - 4, lampY + 5, 2, lampX - 4, lampY + 5, 26);
        poolGrad.addColorStop(0, 'rgba(253, 224, 71, 0.45)');
        poolGrad.addColorStop(1, 'rgba(253, 224, 71, 0)');
        ctx.fillStyle = poolGrad;
        ctx.beginPath();
        ctx.ellipse(lampX - 4, lampY + 5, 26, 13, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.restore();
    },

    drawLogisticsCaravans(ctx, now, eco) {
        if (!this.cityCaravans) {
            this.cityCaravans = [
                { axis: 'x', coord: 4.5, t: 0.1, speed: 0.003, color: '#38bdf8', cargo: '⚡ Power' },
                { axis: 'y', coord: 4.5, t: 0.6, speed: 0.0025, color: '#f59e0b', cargo: '🍞 Food' },
                { axis: 'x', coord: 5.5, t: 0.85, speed: -0.0028, color: '#22c55e', cargo: '📦 Materials' },
            ];
        }

        const isTradeActive = eco?.trade_caravan_active;

        this.cityCaravans.forEach((car, idx) => {
            car.t = (car.t + car.speed + 1) % 1;
            const posAlong = car.t * 8 + 1; // 1 to 9 along road
            const gx = car.axis === 'x' ? posAlong : car.coord;
            const gy = car.axis === 'x' ? car.coord : posAlong;
            const { x: cx, y: cy } = this.toIso(gx, gy);

            ctx.save();
            ctx.translate(cx, cy + 12);

            // Hover shadow
            ctx.fillStyle = 'rgba(0, 0, 0, 0.35)';
            ctx.beginPath();
            ctx.ellipse(0, 4, 10, 5, 0, 0, Math.PI * 2);
            ctx.fill();

            // Hover thruster glow
            ctx.fillStyle = 'rgba(56, 189, 248, 0.65)';
            ctx.beginPath();
            ctx.ellipse(0, 2, 7, 3, 0, 0, Math.PI * 2);
            ctx.fill();

            // Vehicle chassis
            ctx.fillStyle = '#0f172a';
            ctx.fillRect(-8, -8, 16, 8);
            ctx.fillStyle = car.color;
            ctx.fillRect(-6, -13, 12, 6); // Cargo pod
            ctx.fillStyle = '#38bdf8';
            ctx.fillRect(4, -8, 4, 3); // Windshield

            // Floating cargo badge on first truck
            if (idx === 0 || isTradeActive) {
                ctx.fillStyle = 'rgba(15, 23, 42, 0.85)';
                ctx.strokeStyle = car.color;
                ctx.lineWidth = 1;
                ctx.strokeRect(-16, -24, 32, 11);
                ctx.fillRect(-16, -24, 32, 11);
                ctx.fillStyle = '#f8fafc';
                ctx.font = 'bold 8px ' + FONT_SANS;
                ctx.textAlign = 'center';
                ctx.fillText(car.cargo, 0, -16);
            }

            ctx.restore();
        });
    },

    drawSkyDrones(ctx, now) {
        if (!this.cityDrones) {
            this.cityDrones = [
                { x: -140, y: -50, tx: 180, ty: 70, t: 0.15, speed: 0.0018, color: '#38bdf8', cargo: '#22c55e' },
                { x: 160, y: -80, tx: -120, ty: 90, t: 0.55, speed: 0.0014, color: '#f59e0b', cargo: '#0ea5e9' },
                { x: -30, y: 130, tx: 70, ty: -110, t: 0.85, speed: 0.0021, color: '#a855f7', cargo: '#ec4899' },
            ];
        }

        this.cityDrones.forEach(drone => {
            drone.t = (drone.t + drone.speed) % 1;
            const curX = drone.x + (drone.tx - drone.x) * drone.t;
            const curY = drone.y + (drone.ty - drone.y) * drone.t;
            const altitude = -65 + Math.sin(now * 0.003 + drone.t * 10) * 4;

            ctx.save();
            ctx.translate(curX, curY + altitude);

            // Ground shadow
            ctx.fillStyle = 'rgba(0, 0, 0, 0.2)';
            ctx.beginPath();
            ctx.ellipse(0, -altitude + 15, 12, 6, 0, 0, Math.PI * 2);
            ctx.fill();

            // Downward laser scanner guide cone
            const scanGrad = ctx.createLinearGradient(0, 0, 0, -altitude + 15);
            scanGrad.addColorStop(0, `${drone.color}33`);
            scanGrad.addColorStop(1, `${drone.color}00`);
            ctx.fillStyle = scanGrad;
            ctx.beginPath();
            ctx.moveTo(0, 4);
            ctx.lineTo(-10, -altitude + 15);
            ctx.lineTo(10, -altitude + 15);
            ctx.closePath();
            ctx.fill();

            // Drone central chassis
            ctx.fillStyle = '#0f172a';
            ctx.beginPath();
            ctx.roundRect(-6, -3, 12, 6, 2);
            ctx.fill();
            ctx.strokeStyle = drone.color;
            ctx.lineWidth = 1;
            ctx.stroke();

            // 4 Quad-rotor arms & spinning blades
            const rotorRot = now * 0.04;
            const arms = [[-8, -5], [8, -5], [-8, 5], [8, 5]];
            arms.forEach(([ax, ay], i) => {
                ctx.strokeStyle = '#475569';
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.moveTo(0, 0);
                ctx.lineTo(ax, ay);
                ctx.stroke();

                // Spinning rotor blur
                ctx.save();
                ctx.translate(ax, ay);
                ctx.rotate(rotorRot * (i % 2 === 0 ? 1 : -1));
                ctx.strokeStyle = 'rgba(255, 255, 255, 0.6)';
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.moveTo(-5, 0); ctx.lineTo(5, 0);
                ctx.stroke();
                ctx.restore();

                // Navigation LED
                ctx.fillStyle = i < 2 ? '#22c55e' : '#ef4444';
                ctx.fillRect(ax - 0.75, ay - 0.75, 1.5, 1.5);
            });

            // Slung cargo container
            ctx.fillStyle = drone.cargo;
            ctx.fillRect(-3, 3, 6, 5);
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.4)';
            ctx.lineWidth = 0.5;
            ctx.strokeRect(-3, 3, 6, 5);

            ctx.restore();
        });
    },

    drawAtmosphericWeather(ctx, w, h, now, weather, season) {
        if (!this.vfxEnabled) return;

        if (weather === 'rain' || weather === 'storm') {
            ctx.save();
            ctx.strokeStyle = 'rgba(186, 230, 253, 0.42)';
            ctx.lineWidth = 1.3;
            for (let i = 0; i < 40; i++) {
                const rx = ((Math.sin(i * 99 + now * 0.002) * 600) + 600) % 800 - 400;
                const ry = ((i * 37 + now * 0.9) % 600) - 200;
                ctx.beginPath();
                ctx.moveTo(rx, ry);
                ctx.lineTo(rx - 6, ry + 16);
                ctx.stroke();
            }
            // Road puddles and splash rings
            for (let s = 0; s < 10; s++) {
                const sx = ((s * 71 + now * 0.15) % 600) - 300;
                const sy = ((s * 43) % 240) - 40;
                const splashR = (now * 0.025 + s * 4) % 9;
                ctx.strokeStyle = `rgba(186, 230, 253, ${Math.max(0, 0.35 - splashR * 0.035)})`;
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.ellipse(sx, sy, splashR, splashR * 0.5, 0, 0, Math.PI * 2);
                ctx.stroke();
            }
            if (weather === 'storm' && Math.random() < 0.012) {
                // Lightning flash
                ctx.fillStyle = 'rgba(255, 255, 255, 0.35)';
                ctx.fillRect(-w, -h, w * 2, h * 2);
            }
            ctx.restore();
        } else if (season === 'winter' || weather === 'snow') {
            ctx.save();
            ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
            for (let i = 0; i < 45; i++) {
                const sx = ((i * 47) % 800) - 400 + Math.sin(now * 0.003 + i) * 16;
                const sy = ((i * 29 + now * 0.08) % 600) - 200;
                const r = (i % 3 === 0) ? 2.2 : 1.4;
                ctx.beginPath();
                ctx.arc(sx, sy, r, 0, Math.PI * 2);
                ctx.fill();
            }
            ctx.restore();
        } else if (weather === 'drought') {
            ctx.save();
            ctx.fillStyle = 'rgba(245, 158, 11, 0.22)';
            for (let i = 0; i < 30; i++) {
                const dx = ((i * 61 + now * 0.15) % 900) - 450;
                const dy = ((i * 31 + Math.sin(i + now * 0.002) * 40) % 500) - 150;
                ctx.beginPath();
                ctx.arc(dx, dy, 1.8, 0, Math.PI * 2);
                ctx.fill();
            }
            // Heat wave refraction shimmer
            ctx.fillStyle = 'rgba(245, 158, 11, 0.035)';
            for (let hIndex = 0; hIndex < 4; hIndex++) {
                const hy = -60 + hIndex * 45 + Math.sin(now * 0.003 + hIndex) * 10;
                ctx.fillRect(-w, hy, w * 2, 14);
            }
            ctx.restore();
        }
    },

    drawTileStructure(ctx, gx, gy, ix, iy, now, isGhost) {
        if (gx === 4 && gy === 4) {
            // Central Public Plaza Fountain
            ctx.save();
            ctx.translate(ix + 38, iy + 19);
            ctx.fillStyle = '#64748b';
            ctx.beginPath();
            ctx.ellipse(0, 0, 36, 18, 0, 0, Math.PI * 2);
            ctx.fill();
            ctx.strokeStyle = '#94a3b8';
            ctx.lineWidth = 2;
            ctx.stroke();

            ctx.fillStyle = '#0ea5e9';
            ctx.beginPath();
            ctx.ellipse(0, -2, 30, 15, 0, 0, Math.PI * 2);
            ctx.fill();

            const rippleR = ((now * 0.015) % 25);
            ctx.strokeStyle = 'rgba(255,255,255,0.45)';
            ctx.lineWidth = 1.2;
            ctx.beginPath();
            ctx.ellipse(0, -2, rippleR, rippleR * 0.5, 0, 0, Math.PI * 2);
            ctx.stroke();

            ctx.fillStyle = '#cbd5e1';
            ctx.fillRect(-5, -16, 10, 14);

            ctx.fillStyle = '#38bdf8';
            for (let i = 0; i < 6; i++) {
                const angle = (i / 6) * Math.PI * 2 + now * 0.003;
                const dropH = Math.abs(Math.sin(now * 0.006 + i)) * 14;
                const dropX = Math.cos(angle) * (8 + dropH * 0.6);
                const dropY = -16 - dropH + Math.sin(angle) * 4;
                ctx.beginPath();
                ctx.arc(dropX, dropY, 2, 0, Math.PI * 2);
                ctx.fill();
            }
            ctx.restore();
            return;
        }

        // Perimeter Defensive Watchtowers
        if ((gx === 0 && gy === 0) || (gx === 9 && gy === 0) || (gx === 0 && gy === 9) || (gx === 9 && gy === 9)) {
            ctx.save();
            ctx.translate(ix, iy);
            ctx.fillStyle = '#334155';
            ctx.beginPath();
            ctx.moveTo(-12, 10);
            ctx.lineTo(12, 10);
            ctx.lineTo(8, -28);
            ctx.lineTo(-8, -28);
            ctx.closePath();
            ctx.fill();
            ctx.strokeStyle = '#475569';
            ctx.stroke();

            ctx.fillStyle = '#475569';
            ctx.fillRect(-11, -34, 22, 6);

            const flamePulse = Math.sin(now * 0.01 + gx) * 2;
            ctx.fillStyle = '#f59e0b';
            ctx.beginPath();
            ctx.arc(0, -36 + flamePulse, 4, 0, Math.PI * 2);
            ctx.fill();
            ctx.restore();
            return;
        }

        let buildingType = null;
        let buildingLevel = 1;

        if (isGhost) {
            buildingType = this.placementBuilding;
        } else {
            const info = this.getBuildingAtTile(gx, gy);
            if (info) {
                buildingType = info.id;
                buildingLevel = info.level || 1;
            }
        }

        if (!buildingType) return;

        const isNight = this.currentDiurnalHour >= 19.5 || this.currentDiurnalHour < 5.5;
        const isDusk = this.currentDiurnalHour >= 17 && this.currentDiurnalHour < 19.5;
        const windowGlow = isNight || isDusk;

        ctx.save();
        if (isGhost) {
            ctx.globalAlpha = 0.72;
        }

        ctx.translate(ix, iy + 14);

        // 1. VERTICAL FARM
        if (buildingType === 'vertical-farm') {
            ctx.fillStyle = 'rgba(0,0,0,0.3)';
            ctx.beginPath(); ctx.ellipse(0, 0, 18, 9, 0, 0, Math.PI * 2); ctx.fill();

            ctx.fillStyle = '#1e1b4b';
            ctx.beginPath();
            ctx.moveTo(-16, 0); ctx.lineTo(0, 8); ctx.lineTo(0, -48); ctx.lineTo(-16, -56);
            ctx.closePath(); ctx.fill();

            ctx.fillStyle = '#312e81';
            ctx.beginPath();
            ctx.moveTo(0, 8); ctx.lineTo(16, 0); ctx.lineTo(16, -56); ctx.lineTo(0, -48);
            ctx.closePath(); ctx.fill();

            for (let tier = 0; tier < 4; tier++) {
                const ty = -10 - tier * 11;
                ctx.fillStyle = '#a855f7';
                ctx.fillRect(-13, ty - 6, 11, 7);
                ctx.fillStyle = '#22c55e';
                ctx.fillRect(-11, ty - 3, 7, 4);

                ctx.fillStyle = '#c084fc';
                ctx.fillRect(2, ty - 6, 11, 7);
                ctx.fillStyle = '#4ade80';
                ctx.fillRect(4, ty - 3, 7, 4);
            }

            ctx.fillStyle = '#e2e8f0';
            ctx.fillRect(-2, -62, 4, 7);
            const turbineAngle = now * 0.008;
            ctx.save();
            ctx.translate(0, -62);
            ctx.rotate(turbineAngle);
            ctx.strokeStyle = '#38bdf8';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.moveTo(-9, 0); ctx.lineTo(9, 0);
            ctx.moveTo(0, -9); ctx.lineTo(0, 9);
            ctx.stroke();
            ctx.restore();

            // Level 2+: Flanking Aquaponic Greenhouse Wings & Nutrient Feeder Conduits
            if (buildingLevel >= 2) {
                ctx.fillStyle = 'rgba(6, 182, 212, 0.4)';
                ctx.fillRect(-22, -26, 6, 18);
                ctx.fillRect(16, -26, 6, 18);
                ctx.strokeStyle = '#06b6d4';
                ctx.lineWidth = 1;
                ctx.strokeRect(-22, -26, 6, 18);
                ctx.strokeRect(16, -26, 6, 18);

                // Cyan flowing nutrient pipes
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.moveTo(-16, -17); ctx.lineTo(-22, -17);
                ctx.moveTo(16, -17); ctx.lineTo(22, -17);
                ctx.stroke();
            }

            // Level 3: Rooftop Aeroponic Geodesic Biodome & Dual Orbiting Pollinators
            if (buildingLevel >= 3) {
                ctx.fillStyle = 'rgba(56, 189, 248, 0.45)';
                ctx.beginPath();
                ctx.arc(0, -66, 11, Math.PI, 0);
                ctx.fill();
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 1.2;
                ctx.stroke();

                // Geodesic lattice
                ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
                ctx.lineWidth = 0.8;
                ctx.beginPath();
                ctx.moveTo(-8, -66); ctx.lineTo(0, -74); ctx.lineTo(8, -66);
                ctx.stroke();

                // Orbiting micro-pollinator drone
                const pAngle = now * 0.005;
                const px = Math.cos(pAngle) * 16;
                const py = -68 + Math.sin(pAngle) * 6;
                ctx.fillStyle = '#facc15';
                ctx.beginPath(); ctx.arc(px, py, 2, 0, Math.PI * 2); ctx.fill();
            }

        // 2. LIVING WATER GARDEN
        } else if (buildingType === 'water-garden') {
            ctx.fillStyle = 'rgba(0,0,0,0.25)';
            ctx.beginPath(); ctx.ellipse(0, 0, 20, 10, 0, 0, Math.PI * 2); ctx.fill();

            ctx.fillStyle = '#64748b';
            ctx.beginPath(); ctx.ellipse(-9, -16, 7, 4, 0, 0, Math.PI * 2); ctx.fill();
            ctx.fillRect(-16, -16, 14, 18);
            ctx.fillStyle = '#94a3b8';
            ctx.fillRect(-13, -16, 8, 18);

            ctx.fillStyle = '#0284c7';
            ctx.beginPath();
            ctx.ellipse(6, -4, 12, 6, 0, 0, Math.PI * 2);
            ctx.fill();

            ctx.fillStyle = '#10b981';
            for (let r = 0; r < 4; r++) {
                ctx.fillRect(1 + r * 3, -12 - (r % 2) * 3, 2, 7);
            }

            // Level 2+: Stone Aqueduct Arch with Cascading Water Stream & Lotus Pads
            if (buildingLevel >= 2) {
                ctx.strokeStyle = '#94a3b8';
                ctx.lineWidth = 3;
                ctx.beginPath();
                ctx.moveTo(-16, -20); ctx.lineTo(-2, -18); ctx.lineTo(6, -14);
                ctx.stroke();

                // Flowing stream & splash
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 1.5;
                ctx.beginPath();
                ctx.moveTo(6, -14); ctx.lineTo(6, -6);
                ctx.stroke();

                // Water lilies
                ctx.fillStyle = '#f472b6';
                ctx.beginPath(); ctx.arc(10, -5, 2.5, 0, Math.PI * 2); ctx.fill();
                ctx.beginPath(); ctx.arc(3, -2, 2, 0, Math.PI * 2); ctx.fill();
            }

            // Level 3: Crystalline Purifier Spire with Bioluminescent Halo
            if (buildingLevel >= 3) {
                ctx.fillStyle = '#67e8f9';
                ctx.shadowColor = '#06b6d4';
                ctx.shadowBlur = 12;
                ctx.beginPath();
                ctx.moveTo(0, -38); ctx.lineTo(-5, -18); ctx.lineTo(5, -18);
                ctx.closePath();
                ctx.fill();
                ctx.shadowBlur = 0;

                // Glowing ambient halo
                const haloPulse = 14 + Math.sin(now * 0.006) * 3;
                ctx.strokeStyle = 'rgba(6, 182, 212, 0.7)';
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.ellipse(0, -22, haloPulse, haloPulse * 0.5, 0, 0, Math.PI * 2);
                ctx.stroke();
            }

        // 3. SOLAR CANOPY
        } else if (buildingType === 'solar-canopy') {
            ctx.strokeStyle = '#64748b';
            ctx.lineWidth = 2.5;
            ctx.beginPath();
            ctx.moveTo(-14, 2); ctx.lineTo(-14, -14);
            ctx.moveTo(14, 2); ctx.lineTo(14, -14);
            ctx.stroke();

            ctx.fillStyle = '#1e40af';
            ctx.beginPath();
            ctx.moveTo(-18, -14); ctx.lineTo(18, -22); ctx.lineTo(14, -30); ctx.lineTo(-22, -22);
            ctx.closePath(); ctx.fill();
            ctx.strokeStyle = '#60a5fa';
            ctx.lineWidth = 1.2;
            ctx.stroke();

            const glareX = -18 + ((now * 0.03) % 36);
            ctx.strokeStyle = 'rgba(255,255,255,0.6)';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.moveTo(glareX, -16); ctx.lineTo(glareX - 4, -26);
            ctx.stroke();

            // Level 2+: Dual-Axis Sub-Wings & Grid Capacitor Bank
            if (buildingLevel >= 2) {
                ctx.fillStyle = '#2563eb';
                ctx.beginPath();
                ctx.moveTo(-28, -20); ctx.lineTo(-18, -24); ctx.lineTo(-18, -32); ctx.lineTo(-28, -28);
                ctx.closePath(); ctx.fill();
                ctx.beginPath();
                ctx.moveTo(18, -24); ctx.lineTo(28, -28); ctx.lineTo(28, -36); ctx.lineTo(18, -32);
                ctx.closePath(); ctx.fill();

                // Ground battery capacitor
                ctx.fillStyle = '#0f172a';
                ctx.fillRect(-6, -4, 12, 6);
                ctx.fillStyle = '#22c55e';
                ctx.fillRect(-4, -3, 8, 2);
            }

            // Level 3: Quantum Concentrator Levitator & Energy Collector Beams
            if (buildingLevel >= 3) {
                const prismY = -44 + Math.sin(now * 0.005) * 3;
                ctx.fillStyle = '#fde047';
                ctx.shadowColor = '#facc15';
                ctx.shadowBlur = 14;
                ctx.beginPath();
                ctx.moveTo(0, prismY - 6); ctx.lineTo(5, prismY); ctx.lineTo(0, prismY + 6); ctx.lineTo(-5, prismY);
                ctx.closePath(); ctx.fill();
                ctx.shadowBlur = 0;

                // Downward golden optical collector rays
                ctx.strokeStyle = 'rgba(250, 204, 21, 0.45)';
                ctx.lineWidth = 1.5;
                ctx.beginPath();
                ctx.moveTo(0, prismY + 6); ctx.lineTo(-10, -24);
                ctx.moveTo(0, prismY + 6); ctx.lineTo(10, -24);
                ctx.stroke();
            }

        // 4. COURTYARD HOMES
        } else if (buildingType === 'courtyard-homes') {
            ctx.fillStyle = 'rgba(0,0,0,0.25)';
            ctx.beginPath(); ctx.ellipse(0, 0, 18, 9, 0, 0, Math.PI * 2); ctx.fill();

            ctx.fillStyle = '#e2e8f0';
            ctx.beginPath();
            ctx.moveTo(-15, 0); ctx.lineTo(0, 7); ctx.lineTo(15, 0);
            ctx.lineTo(15, -18); ctx.lineTo(0, -25); ctx.lineTo(-15, -18);
            ctx.closePath(); ctx.fill();

            ctx.fillStyle = '#ea580c';
            ctx.beginPath();
            ctx.moveTo(-16, -18); ctx.lineTo(0, -26); ctx.lineTo(16, -18);
            ctx.lineTo(0, -34);
            ctx.closePath(); ctx.fill();

            // Windows
            ctx.fillStyle = windowGlow ? '#fde047' : '#94a3b8';
            if (windowGlow) { ctx.shadowColor = '#facc15'; ctx.shadowBlur = 8; }
            ctx.fillRect(-10, -12, 5, 5);
            ctx.fillRect(4, -12, 5, 5);
            ctx.shadowBlur = 0;

            // Level 2+: Second-Story Timber Sky-Terrace & Hanging Lantern
            if (buildingLevel >= 2) {
                ctx.fillStyle = '#b45309';
                ctx.fillRect(-9, -32, 18, 9);
                ctx.fillStyle = '#9a3412';
                ctx.beginPath();
                ctx.moveTo(-10, -32); ctx.lineTo(0, -39); ctx.lineTo(10, -32);
                ctx.closePath(); ctx.fill();

                // Rooftop planter greens
                ctx.fillStyle = '#22c55e';
                ctx.fillRect(-8, -34, 6, 2);

                // Warm lantern glow
                ctx.fillStyle = '#fde047';
                ctx.beginPath(); ctx.arc(8, -28, 2, 0, Math.PI * 2); ctx.fill();
            }

            // Level 3: Cantilevered Glass Sky-Bridge & Solar Shingle Roof
            if (buildingLevel >= 3) {
                ctx.fillStyle = 'rgba(56, 189, 248, 0.5)';
                ctx.fillRect(-14, -38, 28, 4);
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 1;
                ctx.strokeRect(-14, -38, 28, 4);

                // Solar shingle crown
                ctx.fillStyle = '#0284c7';
                ctx.beginPath();
                ctx.moveTo(-12, -42); ctx.lineTo(0, -48); ctx.lineTo(12, -42);
                ctx.closePath(); ctx.fill();
            }

        // 5. CIVIC LAB
        } else if (buildingType === 'civic-lab') {
            ctx.fillStyle = 'rgba(0,0,0,0.3)';
            ctx.beginPath(); ctx.ellipse(0, 0, 22, 11, 0, 0, Math.PI * 2); ctx.fill();

            ctx.fillStyle = '#94a3b8';
            ctx.fillRect(-18, -6, 36, 8);

            ctx.fillStyle = '#f8fafc';
            for (let c = 0; c < 4; c++) {
                ctx.fillRect(-14 + c * 8, -26, 4, 20);
            }

            ctx.fillStyle = '#cbd5e1';
            ctx.beginPath();
            ctx.moveTo(-18, -26); ctx.lineTo(0, -36); ctx.lineTo(18, -26);
            ctx.closePath(); ctx.fill();

            ctx.fillStyle = '#f59e0b';
            ctx.beginPath();
            ctx.arc(0, -36, 8, Math.PI, 0);
            ctx.fill();

            // Night dome glow
            if (windowGlow) {
                ctx.fillStyle = 'rgba(250, 204, 21, 0.45)';
                ctx.beginPath(); ctx.arc(0, -36, 12, 0, Math.PI * 2); ctx.fill();
            }

            const bannerWave = Math.sin(now * 0.005) * 3;
            ctx.strokeStyle = '#64748b';
            ctx.lineWidth = 1.5;
            ctx.beginPath(); ctx.moveTo(0, -44); ctx.lineTo(0, -56); ctx.stroke();
            ctx.fillStyle = '#ef4444';
            ctx.beginPath();
            ctx.moveTo(0, -56); ctx.lineTo(10 + bannerWave, -52); ctx.lineTo(0, -48);
            ctx.closePath(); ctx.fill();

            // Level 2+: Holographic Planetary Orrery & Flanking Antennas
            if (buildingLevel >= 2) {
                ctx.save();
                ctx.translate(0, -48);
                ctx.rotate(now * 0.004);
                ctx.strokeStyle = 'rgba(56, 189, 248, 0.8)';
                ctx.lineWidth = 1.2;
                ctx.beginPath(); ctx.ellipse(0, 0, 9, 4, 0.4, 0, Math.PI * 2); ctx.stroke();
                ctx.fillStyle = '#38bdf8';
                ctx.beginPath(); ctx.arc(0, 0, 2.5, 0, Math.PI * 2); ctx.fill();
                ctx.restore();

                // Flanking telemetry poles
                ctx.strokeStyle = '#94a3b8';
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.moveTo(-16, -26); ctx.lineTo(-16, -42);
                ctx.moveTo(16, -26); ctx.lineTo(16, -42);
                ctx.stroke();
                ctx.fillStyle = '#10b981';
                ctx.fillRect(-17, -43, 2, 2);
                ctx.fillRect(15, -43, 2, 2);
            }

            // Level 3: Triple Concentric Gyro-Rings & Quantum Civic Zenith Beam
            if (buildingLevel >= 3) {
                ctx.save();
                ctx.translate(0, -48);
                const gPulse = Math.sin(now * 0.006) * 2;
                ctx.strokeStyle = 'rgba(168, 85, 247, 0.85)';
                ctx.lineWidth = 1.4;
                ctx.beginPath(); ctx.ellipse(0, 0, 14 + gPulse, 6, -0.3, 0, Math.PI * 2); ctx.stroke();
                ctx.restore();

                // Upward beacon beam
                ctx.strokeStyle = 'rgba(192, 132, 252, 0.4)';
                ctx.lineWidth = 2;
                ctx.beginPath(); ctx.moveTo(0, -56); ctx.lineTo(0, -84); ctx.stroke();
            }

        // 6. MAKER COOPERATIVE
        } else if (buildingType === 'maker-cooperative') {
            ctx.fillStyle = 'rgba(0,0,0,0.3)';
            ctx.beginPath(); ctx.ellipse(0, 0, 18, 9, 0, 0, Math.PI * 2); ctx.fill();

            ctx.fillStyle = '#991b1b';
            ctx.beginPath();
            ctx.moveTo(-16, 0); ctx.lineTo(0, 7); ctx.lineTo(16, 0);
            ctx.lineTo(16, -20); ctx.lineTo(0, -28); ctx.lineTo(-16, -20);
            ctx.closePath(); ctx.fill();

            ctx.fillStyle = '#475569';
            ctx.beginPath();
            ctx.moveTo(-17, -20); ctx.lineTo(0, -29); ctx.lineTo(17, -20);
            ctx.lineTo(0, -36);
            ctx.closePath(); ctx.fill();

            const furnacePulse = 0.8 + Math.sin(now * 0.01) * 0.2;
            ctx.fillStyle = `rgba(249, 115, 22, ${furnacePulse})`;
            ctx.fillRect(-4, -10, 8, 12);

            ctx.fillStyle = '#7f1d1d';
            ctx.fillRect(8, -42, 6, 24);

            // Level 2+: Articulated Robotic Crane Arm & Heat Exchanger Vents
            if (buildingLevel >= 2) {
                const craneAngle = Math.sin(now * 0.003) * 6;
                ctx.strokeStyle = '#f59e0b';
                ctx.lineWidth = 2;
                ctx.beginPath();
                ctx.moveTo(-10, -25);
                ctx.lineTo(-16 + craneAngle, -42);
                ctx.lineTo(-8 + craneAngle, -42);
                ctx.stroke();

                // Suspended cargo pallet
                ctx.fillStyle = '#d97706';
                ctx.fillRect(-9 + craneAngle, -38, 4, 3);

                // Secondary vent stack
                ctx.fillStyle = '#64748b';
                ctx.fillRect(-14, -34, 4, 14);
            }

            // Level 3: Plasma Induction Crucible & Magnetic Rail Conveyor
            if (buildingLevel >= 3) {
                ctx.fillStyle = '#fff7ed';
                ctx.shadowColor = '#ea580c';
                ctx.shadowBlur = 16;
                ctx.beginPath(); ctx.arc(0, -22, 5, 0, Math.PI * 2); ctx.fill();
                ctx.shadowBlur = 0;

                // Elevated magnetic conveyor line
                ctx.strokeStyle = '#0284c7';
                ctx.lineWidth = 2;
                ctx.beginPath(); ctx.moveTo(-18, -12); ctx.lineTo(-26, -6); ctx.stroke();
                ctx.fillStyle = '#38bdf8';
                ctx.fillRect(-24, -8, 3, 2);
            }

        // 7. ARCOLOGY SPINE (Futuristic Mega-Structure)
        } else if (buildingType === 'arcology-spine') {
            ctx.fillStyle = 'rgba(0,0,0,0.4)';
            ctx.beginPath(); ctx.ellipse(0, 0, 22, 11, 0, 0, Math.PI * 2); ctx.fill();

            // Tower body (Left & Right Facets)
            ctx.fillStyle = '#0f172a';
            ctx.beginPath();
            ctx.moveTo(-16, 0); ctx.lineTo(0, 9); ctx.lineTo(0, -68); ctx.lineTo(-14, -76);
            ctx.closePath(); ctx.fill();

            ctx.fillStyle = '#1e293b';
            ctx.beginPath();
            ctx.moveTo(0, 9); ctx.lineTo(16, 0); ctx.lineTo(14, -76); ctx.lineTo(0, -68);
            ctx.closePath(); ctx.fill();

            // Sky-gardens & Cantilevered Balconies
            ctx.fillStyle = '#0ea5e9';
            ctx.fillRect(-17, -30, 34, 4);
            ctx.fillStyle = '#22c55e';
            ctx.fillRect(-15, -33, 10, 3);
            ctx.fillRect(5, -33, 10, 3);

            ctx.fillStyle = '#0284c7';
            ctx.fillRect(-15, -52, 30, 4);

            // Windows Matrices
            const winColor = windowGlow ? '#38bdf8' : '#475569';
            if (windowGlow) { ctx.shadowColor = '#38bdf8'; ctx.shadowBlur = 10; }
            for (let f = 0; f < 5; f++) {
                const wy = -14 - f * 11;
                ctx.fillStyle = winColor;
                ctx.fillRect(-11, wy, 8, 4);
                ctx.fillStyle = windowGlow ? '#facc15' : '#64748b';
                ctx.fillRect(3, wy, 8, 4);
            }
            ctx.shadowBlur = 0;

            // Spire Peak Beacon
            ctx.strokeStyle = '#38bdf8';
            ctx.lineWidth = 2;
            ctx.beginPath(); ctx.moveTo(0, -74); ctx.lineTo(0, -92); ctx.stroke();
            const beaconAlpha = 0.5 + Math.sin(now * 0.008) * 0.5;
            ctx.fillStyle = `rgba(239, 68, 68, ${beaconAlpha})`;
            ctx.beginPath(); ctx.arc(0, -93, 3, 0, Math.PI * 2); ctx.fill();

            // Level 2+: Glass Observation Pods & Transit Struts
            if (buildingLevel >= 2) {
                // Moving elevator car
                const elevY = -20 - Math.abs(Math.sin(now * 0.003)) * 45;
                ctx.fillStyle = '#38bdf8';
                ctx.fillRect(13, elevY, 3, 5);

                // Cantilevered observation lounge
                ctx.fillStyle = 'rgba(56, 189, 248, 0.4)';
                ctx.fillRect(-22, -45, 8, 5);
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 1;
                ctx.strokeRect(-22, -45, 8, 5);
            }

            // Level 3: Anti-Gravity Halo Stabilization Ring & Quadrant Beacons
            if (buildingLevel >= 3) {
                ctx.strokeStyle = 'rgba(56, 189, 248, 0.9)';
                ctx.lineWidth = 2;
                ctx.shadowColor = '#38bdf8';
                ctx.shadowBlur = 12;
                ctx.beginPath(); ctx.ellipse(0, -84, 18, 6, 0, 0, Math.PI * 2); ctx.stroke();
                ctx.shadowBlur = 0;

                // 4 Navigational corner LEDs
                [[-18, -84], [18, -84], [0, -90], [0, -78]].forEach(([bx, by]) => {
                    ctx.fillStyle = '#22c55e';
                    ctx.beginPath(); ctx.arc(bx, by, 1.5, 0, Math.PI * 2); ctx.fill();
                });
            }

        // 8. TOKAMAK FUSION REACTOR (Clean Fusion Core)
        } else if (buildingType === 'fusion-reactor') {
            ctx.fillStyle = 'rgba(0,0,0,0.35)';
            ctx.beginPath(); ctx.ellipse(0, 0, 24, 12, 0, 0, Math.PI * 2); ctx.fill();

            // Torus Base Chamber
            ctx.fillStyle = '#1e293b';
            ctx.beginPath(); ctx.ellipse(0, -10, 20, 10, 0, 0, Math.PI * 2); ctx.fill();
            ctx.strokeStyle = '#06b6d4';
            ctx.lineWidth = 2;
            ctx.stroke();

            // Magnetic Coils
            for (let c = 0; c < 6; c++) {
                const ca = (c / 6) * Math.PI * 2;
                const cx = Math.cos(ca) * 16;
                const cy = -10 + Math.sin(ca) * 8;
                ctx.fillStyle = '#475569';
                ctx.fillRect(cx - 2, cy - 8, 4, 16);
            }

            // Central Vertical Plasma Column
            const corePulse = Math.sin(now * 0.012) * 3;
            ctx.shadowColor = '#00f2fe';
            ctx.shadowBlur = 18;
            ctx.fillStyle = '#38bdf8';
            ctx.fillRect(-4, -40, 8, 30);
            ctx.fillStyle = '#e879f9';
            ctx.beginPath(); ctx.arc(0, -25, 6 + corePulse, 0, Math.PI * 2); ctx.fill();
            ctx.shadowBlur = 0;

            // Level 2+: Quad Magnetic Boosters & Inter-Coil Energy Arcs
            if (buildingLevel >= 2) {
                ctx.strokeStyle = '#22d3ee';
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.moveTo(-14, -18); ctx.lineTo(-4, -30);
                ctx.moveTo(14, -18); ctx.lineTo(4, -30);
                ctx.stroke();

                // Cryo conduits
                ctx.strokeStyle = '#67e8f9';
                ctx.lineWidth = 1.5;
                ctx.beginPath();
                ctx.arc(0, -10, 23, 0.2, 1.4); ctx.stroke();
                ctx.beginPath();
                ctx.arc(0, -10, 23, 3.4, 4.6); ctx.stroke();
            }

            // Level 3: Dual-Torus Matrix & Swirling Quantum Plasma Orbit
            if (buildingLevel >= 3) {
                const oAngle = now * 0.01;
                const ox = Math.cos(oAngle) * 18;
                const oy = -25 + Math.sin(oAngle) * 7;
                ctx.fillStyle = '#f472b6';
                ctx.shadowColor = '#e879f9';
                ctx.shadowBlur = 14;
                ctx.beginPath(); ctx.arc(ox, oy, 3.5, 0, Math.PI * 2); ctx.fill();
                ctx.beginPath(); ctx.arc(-ox, -25 - Math.sin(oAngle) * 7, 3.5, 0, Math.PI * 2); ctx.fill();
                ctx.shadowBlur = 0;
            }

        // 9. NANOTECH FOUNDRY (Molecular Fabrication Cube)
        } else if (buildingType === 'nanotech-foundry') {
            ctx.fillStyle = 'rgba(0,0,0,0.35)';
            ctx.beginPath(); ctx.ellipse(0, 0, 20, 10, 0, 0, Math.PI * 2); ctx.fill();

            // Obsidian Cube
            ctx.fillStyle = '#090d16';
            ctx.beginPath();
            ctx.moveTo(-16, 0); ctx.lineTo(0, 8); ctx.lineTo(0, -28); ctx.lineTo(-16, -36);
            ctx.closePath(); ctx.fill();
            ctx.fillStyle = '#111827';
            ctx.beginPath();
            ctx.moveTo(0, 8); ctx.lineTo(16, 0); ctx.lineTo(16, -36); ctx.lineTo(0, -28);
            ctx.closePath(); ctx.fill();

            // Neon Circuit Inlays
            ctx.strokeStyle = '#10b981';
            ctx.lineWidth = 1.2;
            ctx.beginPath();
            ctx.moveTo(-12, -8); ctx.lineTo(-4, -12); ctx.lineTo(-4, -24);
            ctx.moveTo(4, -12); ctx.lineTo(12, -8); ctx.lineTo(12, -24);
            ctx.stroke();

            // Central Levitating Nanite Crystal
            const crystalY = -42 + Math.sin(now * 0.006) * 3;
            ctx.fillStyle = '#34d399';
            ctx.shadowColor = '#10b981';
            ctx.shadowBlur = 14;
            ctx.beginPath();
            ctx.moveTo(0, crystalY - 8); ctx.lineTo(6, crystalY); ctx.lineTo(0, crystalY + 8); ctx.lineTo(-6, crystalY);
            ctx.closePath(); ctx.fill();
            ctx.shadowBlur = 0;

            // Level 2+: Deposition Gantry Arms & Holographic Wireframe Cube
            if (buildingLevel >= 2) {
                ctx.strokeStyle = '#a855f7';
                ctx.lineWidth = 1;
                ctx.strokeRect(-8, crystalY - 14, 16, 12);

                // Laser deposition beams
                ctx.strokeStyle = 'rgba(16, 185, 129, 0.7)';
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.moveTo(-14, -20); ctx.lineTo(0, crystalY);
                ctx.moveTo(14, -20); ctx.lineTo(0, crystalY);
                ctx.stroke();
            }

            // Level 3: Tri-Crystal Molecular Lattice & Nanite Particle Swarm
            if (buildingLevel >= 3) {
                for (let k = 0; k < 3; k++) {
                    const cAngle = (k / 3) * Math.PI * 2 + now * 0.004;
                    const cx = Math.cos(cAngle) * 12;
                    const cy = crystalY + Math.sin(cAngle) * 5;
                    ctx.fillStyle = '#6ee7b7';
                    ctx.beginPath(); ctx.arc(cx, cy, 2, 0, Math.PI * 2); ctx.fill();
                }
            }

        // 10. HYPERLOOP LOGISTICS TERMINAL
        } else if (buildingType === 'hyperloop-exchange') {
            ctx.fillStyle = 'rgba(0,0,0,0.3)';
            ctx.beginPath(); ctx.ellipse(0, 0, 22, 11, 0, 0, Math.PI * 2); ctx.fill();

            // Sleek Terminal Dome
            ctx.fillStyle = '#1e3a8a';
            ctx.beginPath(); ctx.arc(0, -10, 16, Math.PI, 0); ctx.fill();
            ctx.strokeStyle = '#60a5fa'; ctx.lineWidth = 1.5; ctx.stroke();

            // Twin Sub-surface Vacuum Tubes
            ctx.fillStyle = '#475569';
            ctx.fillRect(-22, -8, 10, 6);
            ctx.fillRect(12, -8, 10, 6);
            ctx.fillStyle = '#38bdf8';
            ctx.fillRect(-22, -6, 10, 2);
            ctx.fillRect(12, -6, 10, 2);

            // Hover Freight Pod
            const podX = Math.sin(now * 0.005) * 6;
            ctx.fillStyle = '#f59e0b';
            ctx.fillRect(podX - 6, -18, 12, 6);
            ctx.fillStyle = 'rgba(56, 189, 248, 0.6)';
            ctx.beginPath(); ctx.ellipse(podX, -11, 7, 2, 0, 0, Math.PI * 2); ctx.fill();

            // Level 2+: Elevated Supersonic Departure Track & Airlocks
            if (buildingLevel >= 2) {
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 2;
                ctx.beginPath();
                ctx.arc(0, -18, 22, Math.PI * 0.8, Math.PI * 0.2, true);
                ctx.stroke();

                // Boarding airlocks
                ctx.fillStyle = '#10b981';
                ctx.fillRect(-15, -12, 3, 4);
                ctx.fillRect(12, -12, 3, 4);
            }

            // Level 3: 360° Transit Interchange Tube & Sonic Wave Ring
            if (buildingLevel >= 3) {
                const sRing = ((now * 0.02) % 24);
                ctx.strokeStyle = `rgba(56, 189, 248, ${Math.max(0, 1 - sRing / 24)})`;
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.ellipse(0, -10, sRing, sRing * 0.5, 0, 0, Math.PI * 2);
                ctx.stroke();
            }

        // 11. QUANTUM TELEMETRY ARRAY
        } else if (buildingType === 'quantum-observatory') {
            ctx.fillStyle = 'rgba(0,0,0,0.35)';
            ctx.beginPath(); ctx.ellipse(0, 0, 20, 10, 0, 0, Math.PI * 2); ctx.fill();

            // Tiered Pedestal
            ctx.fillStyle = '#334155';
            ctx.fillRect(-12, -12, 24, 14);

            // Parabolic Dishes
            ctx.strokeStyle = '#cbd5e1';
            ctx.lineWidth = 2.5;
            ctx.beginPath(); ctx.arc(-7, -24, 9, 0.4, 2.6); ctx.stroke();
            ctx.beginPath(); ctx.arc(7, -24, 9, 0.4, 2.6); ctx.stroke();

            // Laser Telemetry Column
            ctx.strokeStyle = 'rgba(168, 85, 247, 0.8)';
            ctx.lineWidth = 2;
            ctx.shadowColor = '#c084fc';
            ctx.shadowBlur = 12;
            ctx.beginPath(); ctx.moveTo(0, -20); ctx.lineTo(0, -60); ctx.stroke();
            ctx.shadowBlur = 0;

            // Level 2+: Tri-Dish Phased Array & Holographic Star Chart
            if (buildingLevel >= 2) {
                ctx.strokeStyle = '#94a3b8';
                ctx.lineWidth = 2;
                ctx.beginPath(); ctx.arc(0, -34, 7, 0.4, 2.6); ctx.stroke();

                // Holographic star chart sphere
                ctx.strokeStyle = 'rgba(192, 132, 252, 0.6)';
                ctx.lineWidth = 1;
                ctx.beginPath(); ctx.arc(0, -48, 8, 0, Math.PI * 2); ctx.stroke();
                ctx.fillStyle = '#38bdf8';
                ctx.beginPath(); ctx.arc(3, -50, 1.5, 0, Math.PI * 2); ctx.fill();
                ctx.beginPath(); ctx.arc(-4, -46, 1.5, 0, Math.PI * 2); ctx.fill();
            }

            // Level 3: Deep-Space Subspace Emitter & Reconnaissance Sensors
            if (buildingLevel >= 3) {
                ctx.strokeStyle = '#ec4899';
                ctx.shadowColor = '#f43f5e';
                ctx.shadowBlur = 16;
                ctx.lineWidth = 2.5;
                ctx.beginPath(); ctx.moveTo(0, -60); ctx.lineTo(0, -90); ctx.stroke();
                ctx.shadowBlur = 0;

                // Orbiting sensor probe
                const probeAngle = now * 0.007;
                ctx.fillStyle = '#fbbf24';
                ctx.beginPath();
                ctx.arc(Math.cos(probeAngle) * 14, -60 + Math.sin(probeAngle) * 5, 2, 0, Math.PI * 2);
                ctx.fill();
            }

        // 12. STORY FORUM (Civic Agorá & Eternal Flame)
        } else if (buildingType === 'story-forum') {
            ctx.fillStyle = 'rgba(0,0,0,0.3)';
            ctx.beginPath(); ctx.ellipse(0, 0, 22, 11, 0, 0, Math.PI * 2); ctx.fill();

            // Stepped Circular Agorá
            ctx.fillStyle = '#fef08a';
            ctx.beginPath(); ctx.ellipse(0, -4, 18, 9, 0, 0, Math.PI * 2); ctx.fill();
            ctx.fillStyle = '#fde047';
            ctx.beginPath(); ctx.ellipse(0, -8, 12, 6, 0, 0, Math.PI * 2); ctx.fill();

            // Central Brazier with Eternal Flame
            ctx.fillStyle = '#475569';
            ctx.fillRect(-3, -13, 6, 6);
            const flameH = 6 + Math.sin(now * 0.015) * 3;
            ctx.fillStyle = '#ef4444';
            ctx.beginPath(); ctx.moveTo(-3, -13); ctx.lineTo(0, -13 - flameH); ctx.lineTo(3, -13); ctx.closePath(); ctx.fill();
            ctx.fillStyle = '#f59e0b';
            ctx.beginPath(); ctx.arc(0, -13 - flameH * 0.4, 2, 0, Math.PI * 2); ctx.fill();

            // Canopy Sails
            ctx.strokeStyle = '#f97316';
            ctx.lineWidth = 1.5;
            ctx.beginPath(); ctx.moveTo(-16, -6); ctx.lineTo(-12, -26); ctx.lineTo(0, -28); ctx.stroke();
            ctx.beginPath(); ctx.moveTo(16, -6); ctx.lineTo(12, -26); ctx.lineTo(0, -28); ctx.stroke();

            // Level 2+: Ring of Marble Colonnades & Dual Ceremonial Braziers
            if (buildingLevel >= 2) {
                ctx.fillStyle = '#f8fafc';
                for (let c = 0; c < 5; c++) {
                    const ca = (c / 4) * Math.PI;
                    const cx = Math.cos(ca) * 16;
                    const cy = -6 + Math.sin(ca) * 6;
                    ctx.fillRect(cx - 1, cy - 12, 2, 12);
                }

                // Dual braziers
                ctx.fillStyle = '#f59e0b';
                ctx.beginPath(); ctx.arc(-14, -7, 2, 0, Math.PI * 2); ctx.fill();
                ctx.beginPath(); ctx.arc(14, -7, 2, 0, Math.PI * 2); ctx.fill();
            }

            // Level 3: Floating Holographic Mythogram Halo
            if (buildingLevel >= 3) {
                ctx.strokeStyle = 'rgba(250, 204, 21, 0.75)';
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.ellipse(0, -36, 14, 5, 0, 0, Math.PI * 2);
                ctx.stroke();
                ctx.fillStyle = '#fbbf24';
                ctx.font = '7px ' + FONT_MONO;
                ctx.textAlign = 'center';
                ctx.fillText('✦ 🏛️ ✦', 0, -35);
            }

        // 13. GRAND AMPHITHEATER (Civic Monument)
        } else if (buildingType === 'grand-amphitheater') {
            ctx.fillStyle = 'rgba(0,0,0,0.4)';
            ctx.beginPath(); ctx.ellipse(0, 0, 24, 12, 0, 0, Math.PI * 2); ctx.fill();

            // Layered Semicircular Colosseum
            ctx.fillStyle = '#cbd5e1';
            ctx.beginPath(); ctx.arc(0, -8, 20, Math.PI, 0); ctx.fill();
            ctx.fillStyle = '#e2e8f0';
            ctx.beginPath(); ctx.arc(0, -12, 14, Math.PI, 0); ctx.fill();

            // Holographic Stage Spotlight
            ctx.fillStyle = 'rgba(236, 72, 153, 0.25)';
            ctx.beginPath(); ctx.moveTo(0, -8); ctx.lineTo(-14, -36); ctx.lineTo(14, -36); ctx.closePath(); ctx.fill();
            ctx.fillStyle = '#ec4899';
            ctx.beginPath(); ctx.arc(0, -8, 4, 0, Math.PI * 2); ctx.fill();

            // Festal Banners
            ctx.strokeStyle = '#8b5cf6'; ctx.lineWidth = 1.5;
            ctx.beginPath(); ctx.moveTo(-18, -12); ctx.lineTo(-18, -26); ctx.stroke();
            ctx.beginPath(); ctx.moveTo(18, -12); ctx.lineTo(18, -26); ctx.stroke();

            // Level 2+: Acoustic Shell Arcs & Grand Plaza Obelisks
            if (buildingLevel >= 2) {
                ctx.strokeStyle = 'rgba(236, 72, 153, 0.6)';
                ctx.lineWidth = 1.5;
                ctx.beginPath(); ctx.arc(0, -14, 22, Math.PI * 0.85, Math.PI * 0.15, true); ctx.stroke();

                // Entrance plaza obelisks
                ctx.fillStyle = '#94a3b8';
                ctx.fillRect(-22, -18, 3, 14);
                ctx.fillRect(19, -18, 3, 14);
                ctx.fillStyle = '#38bdf8';
                ctx.fillRect(-22, -20, 3, 2);
                ctx.fillRect(19, -20, 3, 2);
            }

            // Level 3: Celestial Stage Aurora Projection Canopy
            if (buildingLevel >= 3) {
                const aShift = Math.sin(now * 0.005) * 6;
                const aGrad = ctx.createLinearGradient(-15, -42, 15, -42);
                aGrad.addColorStop(0, 'rgba(168, 85, 247, 0.4)');
                aGrad.addColorStop(0.5, 'rgba(236, 72, 153, 0.6)');
                aGrad.addColorStop(1, 'rgba(56, 189, 248, 0.4)');
                ctx.fillStyle = aGrad;
                ctx.beginPath();
                ctx.moveTo(-18, -36); ctx.lineTo(aShift, -52); ctx.lineTo(18, -36);
                ctx.closePath(); ctx.fill();
            }
        }

        // Structural Level Badges (Tactical Holographic Capsule Pill)
        if (!isGhost && buildingLevel >= 2) {
            const isMax = buildingLevel >= 3;
            const badgeY = -56;
            const pillW = isMax ? 52 : 44;
            const pillH = 14;

            ctx.save();
            ctx.shadowColor = isMax ? 'rgba(250, 204, 21, 0.65)' : 'rgba(56, 189, 248, 0.65)';
            ctx.shadowBlur = 8;
            ctx.fillStyle = 'rgba(11, 19, 38, 0.94)';
            ctx.beginPath();
            ctx.roundRect(-pillW / 2, badgeY - pillH / 2, pillW, pillH, pillH / 2);
            ctx.fill();

            ctx.strokeStyle = isMax ? '#facc15' : '#38bdf8';
            ctx.lineWidth = 1.2;
            ctx.stroke();

            ctx.shadowBlur = 0;
            ctx.font = 'bold 8.5px ' + FONT_MONO;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = isMax ? '#fef08a' : '#bae6fd';
            const badgeLabel = isMax ? 'LVL 3 ★' : 'LVL 2 ★';
            ctx.fillText(badgeLabel, 0, badgeY + 0.5);
            ctx.restore();
        }

        ctx.restore();
    },

    drawChimneySmoke(ctx, now) {
        const { x: sx, y: sy } = this.toIso(2, 7);
        ctx.save();
        ctx.translate(sx + 11, sy - 28);
        this.chimneySmoke.forEach(p => {
            p.offsetY += p.vy;
            p.offsetX += p.vx;
            p.alpha -= 0.006;
            if (p.alpha <= 0) {
                p.offsetY = 0;
                p.offsetX = (Math.random() - 0.5) * 2;
                p.alpha = 0.65;
            }
            ctx.fillStyle = `rgba(226, 232, 240, ${Math.max(0, p.alpha)})`;
            ctx.beginPath();
            ctx.arc(p.offsetX, p.offsetY, p.r, 0, Math.PI * 2);
            ctx.fill();
        });
        ctx.restore();
    },

    drawCitizens(ctx, now) {
        this.cityCitizens.forEach(c => {
            c.bob += 0.12;
            if (c.axis === 'x') {
                c.gx += c.speed * c.dir;
                if (c.gx >= 8) { c.dir = -1; }
                else if (c.gx <= 1) { c.dir = 1; }
            } else {
                c.gy += c.speed * c.dir;
                if (c.gy >= 8) { c.dir = -1; }
                else if (c.gy <= 1) { c.dir = 1; }
            }

            const { x: cx, y: cy } = this.toIso(c.gx, c.gy);
            const bobY = Math.abs(Math.sin(c.bob)) * 3;

            ctx.save();
            ctx.translate(cx, cy + 12 - bobY);

            ctx.fillStyle = 'rgba(0,0,0,0.3)';
            ctx.beginPath();
            ctx.ellipse(0, bobY, 4, 2, 0, 0, Math.PI * 2);
            ctx.fill();

            ctx.strokeStyle = '#1e293b';
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            ctx.moveTo(-1.5, 0); ctx.lineTo(-1.5, 5);
            ctx.moveTo(1.5, 0); ctx.lineTo(1.5, 5);
            ctx.stroke();

            ctx.fillStyle = c.color;
            ctx.fillRect(-2.5, -5, 5, 5);

            ctx.fillStyle = '#fed7aa';
            ctx.beginPath();
            ctx.arc(0, -7.5, 2.5, 0, Math.PI * 2);
            ctx.fill();

            c.bubbleTimer += 0.5;
            if (c.bubbleTimer > 250 && c.bubbleTimer < 320) {
                ctx.fillStyle = 'rgba(15, 23, 42, 0.88)';
                ctx.fillRect(-12, -26, 24, 15);
                ctx.strokeStyle = '#38bdf8';
                ctx.lineWidth = 1;
                ctx.strokeRect(-12, -26, 24, 15);
                ctx.font = '10px ' + FONT_SANS;
                ctx.textAlign = 'center';
                ctx.fillText(c.emoji, 0, -15);
            } else if (c.bubbleTimer >= 320) {
                c.bubbleTimer = Math.random() * 100;
            }

            ctx.restore();
        });
    },

    drawRaiders(ctx, now, threat) {
        const raiders = [
            { gx: -0.8, gy: 4.5, angle: 0.2 },
            { gx: 9.8, gy: 5.2, angle: -0.4 }
        ];

        raiders.forEach((r) => {
            const { x: rx, y: ry } = this.toIso(r.gx, r.gy);
            ctx.save();
            ctx.translate(rx, ry + 16);
            ctx.rotate(r.angle);

            ctx.fillStyle = '#78350f';
            ctx.fillRect(-10, -6, 20, 12);
            ctx.fillStyle = '#18181b';
            ctx.fillRect(-12, -8, 5, 4);
            ctx.fillRect(7, -8, 5, 4);
            ctx.fillRect(-12, 4, 5, 4);
            ctx.fillRect(7, 4, 5, 4);

            ctx.strokeStyle = '#ef4444';
            ctx.lineWidth = 1.5;
            ctx.beginPath(); ctx.moveTo(0, -6); ctx.lineTo(0, -18); ctx.stroke();
            ctx.fillStyle = '#b91c1c';
            ctx.fillRect(0, -18, 8, 5);

            ctx.fillStyle = 'rgba(239, 68, 68, 0.2)';
            ctx.beginPath();
            ctx.moveTo(10, 0); ctx.lineTo(40, -15); ctx.lineTo(40, 15);
            ctx.closePath();
            ctx.fill();

            ctx.restore();
        });
    },

    drawShieldDome(ctx, now) {
        const { x: cx, y: cy } = this.toIso(4.5, 4.5);
        ctx.save();
        ctx.translate(cx, cy);

        const radiusX = 280;
        const radiusY = 175;

        const domeGrad = ctx.createRadialGradient(0, -40, 20, 0, -20, radiusX);
        domeGrad.addColorStop(0, 'rgba(56, 189, 248, 0.28)');
        domeGrad.addColorStop(0.7, 'rgba(6, 182, 212, 0.12)');
        domeGrad.addColorStop(1, 'rgba(14, 165, 233, 0.02)');
        ctx.fillStyle = domeGrad;
        ctx.beginPath();
        ctx.ellipse(0, -40, radiusX, radiusY, 0, Math.PI, 0);
        ctx.fill();

        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 2.5;
        ctx.shadowColor = '#00f2fe';
        ctx.shadowBlur = 16;
        ctx.beginPath();
        ctx.ellipse(0, -40, radiusX, radiusY, 0, Math.PI, 0);
        ctx.stroke();
        ctx.shadowBlur = 0;

        ctx.strokeStyle = 'rgba(56, 189, 248, 0.35)';
        ctx.lineWidth = 1.2;
        for (let ring = 1; ring <= 4; ring++) {
            const rx = radiusX * (ring / 5);
            const ry = radiusY * (ring / 5);
            ctx.beginPath();
            ctx.ellipse(0, -40, rx, ry, 0, Math.PI, 0);
            ctx.stroke();
        }

        const corners = [this.toIso(0, 0), this.toIso(9, 0), this.toIso(0, 9), this.toIso(9, 9)];
        ctx.strokeStyle = 'rgba(56, 189, 248, 0.6)';
        ctx.lineWidth = 1.8;
        corners.forEach(cor => {
            ctx.beginPath();
            ctx.moveTo(cor.x - cx, cor.y - cy - 20);
            ctx.lineTo(0, -40 - radiusY + 10);
            ctx.stroke();
        });

        ctx.restore();
    },

    renderRealmView(ctx, w, h, now) {
        const cx = 0;
        const cy = 0;

        ctx.save();
        ctx.strokeStyle = 'rgba(241, 185, 107, 0.12)';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.arc(cx, cy, 260, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([6, 6]);
        ctx.beginPath();
        ctx.arc(cx, cy, 180, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.restore();

        const realms = [
            { id: 'sanctuary-haven', name: 'Sanctuary Haven', title: 'Your Sovereign City', power: 'Metropolis', stance: 'ally', x: 0, y: 0, crest: '🏰', color: '#f59e0b', r: 52 },
            { id: 'barony-oakhaven', name: 'Barony of Oakhaven', title: 'Martial Fiefdom · Baron Kaelen', power: 'Power 125 · Feudal Levies', stance: 'neutral', x: 0, y: -190, crest: '🛡️', color: '#eab308', r: 44 },
            { id: 'riverside-vassal', name: 'Riverside Protectorate', title: 'Agricultural Vassal · Gov. Chen', power: 'Tribute: +100 Food, +80 Water', stance: 'vassal', x: -260, y: -30, crest: '🌾', color: '#10b981', r: 44 },
            { id: 'iron-mandate', name: 'Iron Mandate Hegemony', title: 'Imperial Hegemon · Arch-Imperator', power: 'Power 290 · Threat Aura', stance: 'hostile', x: 260, y: -30, crest: '🦅', color: '#ef4444', r: 44 },
            { id: 'mercantile-league', name: 'Free Mercantile League', title: 'Trade Coalition · Chancellor Mirren', power: 'Coalition Pact · Credit Lines', stance: 'coalition', x: 120, y: 190, crest: '⚖️', color: '#38bdf8', r: 44 },
            { id: 'dust-canyon-raiders', name: 'Dust Canyon Raiders', title: 'Desert Insurgency · Warlord Jax', power: 'Warlord Raids · High Incursion', stance: 'hostile', x: -180, y: 170, crest: '⚔️', color: '#f97316', r: 44 },
        ];

        realms.slice(1).forEach(r => {
            ctx.save();
            ctx.strokeStyle = r.color;
            ctx.globalAlpha = 0.35;
            ctx.lineWidth = 2;
            ctx.setLineDash([5, 5]);
            ctx.beginPath();
            ctx.moveTo(0, 0);
            ctx.lineTo(r.x, r.y);
            ctx.stroke();

            const packetT = ((now * 0.0006 + r.x * 0.01) % 1);
            const px = r.x * packetT;
            const py = r.y * packetT;
            ctx.fillStyle = r.color;
            ctx.globalAlpha = 0.9;
            ctx.beginPath();
            ctx.arc(px, py, 3.5, 0, Math.PI * 2);
            ctx.fill();
            ctx.restore();
        });

        realms.forEach(r => {
            const isHovered = this.hoveredRealmEntity?.id === r.id;
            ctx.save();
            ctx.translate(r.x, r.y);

            if (isHovered) {
                ctx.fillStyle = `${r.color}22`;
                ctx.beginPath();
                ctx.arc(0, 0, r.r + 12, 0, Math.PI * 2);
                ctx.fill();
            }

            ctx.fillStyle = '#0f172a';
            ctx.beginPath();
            ctx.arc(0, 0, r.r, 0, Math.PI * 2);
            ctx.fill();
            ctx.strokeStyle = isHovered ? '#ffffff' : r.color;
            ctx.lineWidth = isHovered ? 3 : 2;
            ctx.stroke();

            ctx.font = '22px ' + FONT_SANS;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(r.crest, 0, -4);

            ctx.fillStyle = '#f8fafc';
            ctx.font = 'bold 12px ' + FONT_SANS;
            ctx.fillText(r.name, 0, r.r + 16);

            ctx.fillStyle = 'var(--text-muted, #94a3b8)';
            ctx.font = '10px ' + FONT_SANS;
            ctx.fillText(r.title, 0, r.r + 30);

            ctx.fillStyle = r.color;
            ctx.font = 'bold 9px ' + FONT_MONO;
            ctx.fillText(r.power, 0, r.r + 43);

            ctx.restore();
        });
    },

    renderWildernessEcosystemView(ctx, w, h, now) {
        ctx.save();
        ctx.translate(0, -60);

        // 1. Lush rolling wilderness terrain
        const terrainGrad = ctx.createLinearGradient(0, -300, 0, 300);
        terrainGrad.addColorStop(0, '#14532d');
        terrainGrad.addColorStop(0.5, '#166534');
        terrainGrad.addColorStop(1, '#15803d');
        ctx.fillStyle = terrainGrad;
        ctx.beginPath();
        ctx.ellipse(0, 0, 480, 260, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = 'rgba(74, 222, 128, 0.25)';
        ctx.lineWidth = 2;
        ctx.stroke();

        // 2. Winding River
        ctx.strokeStyle = '#0284c7';
        ctx.lineWidth = 36;
        ctx.lineCap = 'round';
        ctx.beginPath();
        ctx.moveTo(-360, -180);
        ctx.bezierCurveTo(-150, -40, -50, -140, 60, 20);
        ctx.bezierCurveTo(150, 140, 280, 80, 380, 180);
        ctx.stroke();

        // River specular shimmer
        ctx.strokeStyle = 'rgba(186, 230, 253, 0.45)';
        ctx.lineWidth = 6;
        ctx.beginPath();
        ctx.moveTo(-350, -180);
        ctx.bezierCurveTo(-145, -40, -45, -140, 65, 20);
        ctx.bezierCurveTo(155, 140, 285, 80, 375, 180);
        ctx.stroke();

        // 3. Pine groves & Deciduous forest clumps
        const groves = [
            { x: -240, y: -90, count: 6, color: '#064e3b' },
            { x: -180, y: 110, count: 5, color: '#065f46' },
            { x: 220, y: -120, count: 7, color: '#047857' },
            { x: 190, y: 60, count: 6, color: '#064e3b' },
        ];
        groves.forEach(g => {
            for (let t = 0; t < g.count; t++) {
                const tx = g.x + Math.sin(t * 3) * 32;
                const ty = g.y + Math.cos(t * 4) * 20;
                ctx.fillStyle = '#78350f';
                ctx.fillRect(tx - 2, ty, 4, 10);
                ctx.fillStyle = g.color;
                ctx.beginPath();
                ctx.moveTo(tx - 12, ty);
                ctx.lineTo(tx, ty - 22);
                ctx.lineTo(tx + 12, ty);
                ctx.closePath();
                ctx.fill();
            }
        });

        // 4. Herbivore Deer Herd
        if (!this.deerEntities) {
            this.deerEntities = Array.from({ length: 8 }, (_, i) => ({
                x: -120 + Math.random() * 140,
                y: -40 + Math.random() * 120,
                vx: (Math.random() - 0.5) * 0.4,
                vy: (Math.random() - 0.5) * 0.3,
                bob: Math.random() * 10,
                isBuck: i === 0,
            }));
        }

        this.deerEntities.forEach(d => {
            d.x += d.vx;
            d.y += d.vy;
            d.bob += 0.05;
            if (d.x < -180) { d.x = -180; d.vx *= -1; }
            if (d.x > 80) { d.x = 80; d.vx *= -1; }
            if (d.y < -100) { d.y = -100; d.vy *= -1; }
            if (d.y > 140) { d.y = 140; d.vy *= -1; }

            ctx.save();
            ctx.translate(d.x, d.y);

            // Shadow
            ctx.fillStyle = 'rgba(0, 0, 0, 0.25)';
            ctx.beginPath(); ctx.ellipse(0, 8, 8, 4, 0, 0, Math.PI * 2); ctx.fill();

            // Legs
            ctx.strokeStyle = '#78350f';
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            ctx.moveTo(-4, 0); ctx.lineTo(-4, 9);
            ctx.moveTo(4, 0); ctx.lineTo(4, 9);
            ctx.stroke();

            // Body
            ctx.fillStyle = '#b45309';
            ctx.beginPath(); ctx.ellipse(0, 0, 9, 5, 0, 0, Math.PI * 2); ctx.fill();
            ctx.fillStyle = '#fef3c7';
            ctx.fillRect(7, -2, 3, 3);

            // Head
            const headBob = Math.sin(d.bob) * 3;
            ctx.fillStyle = '#b45309';
            ctx.beginPath();
            ctx.arc(-8, -4 + headBob, 3.5, 0, Math.PI * 2);
            ctx.fill();

            // Antlers
            if (d.isBuck) {
                ctx.strokeStyle = '#451a03';
                ctx.lineWidth = 1.2;
                ctx.beginPath();
                ctx.moveTo(-9, -7 + headBob); ctx.lineTo(-12, -14 + headBob);
                ctx.moveTo(-9, -7 + headBob); ctx.lineTo(-7, -14 + headBob);
                ctx.stroke();
            }

            ctx.restore();
        });

        // 5. Carnivore Wolf Pack
        if (!this.wolfEntities) {
            this.wolfEntities = Array.from({ length: 4 }, (_, i) => ({
                x: 100 + Math.random() * 120,
                y: -60 + Math.random() * 100,
                vx: (Math.random() - 0.5) * 0.5,
                vy: (Math.random() - 0.5) * 0.35,
                prowl: Math.random() * 10,
                alpha: i === 0,
            }));
        }

        this.wolfEntities.forEach(w => {
            w.x += w.vx;
            w.y += w.vy;
            w.prowl += 0.08;
            if (w.x < 40) { w.x = 40; w.vx *= -1; }
            if (w.x > 260) { w.x = 260; w.vx *= -1; }
            if (w.y < -120) { w.y = -120; w.vy *= -1; }
            if (w.y > 120) { w.y = 120; w.vy *= -1; }

            ctx.save();
            ctx.translate(w.x, w.y);

            // Shadow
            ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
            ctx.beginPath(); ctx.ellipse(0, 7, 7, 3.5, 0, 0, Math.PI * 2); ctx.fill();

            // Body
            ctx.fillStyle = w.alpha ? '#1e293b' : '#334155';
            ctx.beginPath(); ctx.ellipse(0, 0, 8, 4.5, 0, 0, Math.PI * 2); ctx.fill();

            // Head
            ctx.beginPath(); ctx.arc(-7, -1, 3.2, 0, Math.PI * 2); ctx.fill();
            ctx.beginPath();
            ctx.moveTo(-8, -4); ctx.lineTo(-10, -7); ctx.lineTo(-6, -4);
            ctx.fill();

            // Glowing Amber Eyes
            ctx.fillStyle = '#f59e0b';
            ctx.fillRect(-8, -2, 1.5, 1.5);

            // Tail
            ctx.strokeStyle = w.alpha ? '#1e293b' : '#334155';
            ctx.lineWidth = 2;
            ctx.beginPath(); ctx.moveTo(6, 0); ctx.lineTo(11, 4); ctx.stroke();

            ctx.restore();
        });

        // 6. Tactical Wilderness HUD Chip
        ctx.fillStyle = 'rgba(15, 23, 42, 0.88)';
        ctx.strokeStyle = '#4ade80';
        ctx.lineWidth = 1.5;
        ctx.strokeRect(-160, 190, 320, 34);
        ctx.fillRect(-160, 190, 320, 34);

        ctx.fillStyle = '#f8fafc';
        ctx.font = 'bold 11px ' + FONT_SANS;
        ctx.textAlign = 'center';
        ctx.fillText('🌿 LIVING WILDERNESS ECOSYSTEM · PREDATOR-PREY WEB', 0, 205);
        ctx.font = '10px ' + FONT_SANS;
        ctx.fillStyle = '#86efac';
        ctx.fillText('🦌 8-12 Herbivore Deer  ·  🐺 4-6 Wolf Pack Hunters  ·  🌾 Savanna Biomass', 0, 218);

        ctx.restore();
    },

    renderCoastalResilienceView(ctx, w, h, now) {
        ctx.save();
        ctx.translate(0, -50);

        // 1. Deep Ocean & Coastal Waters
        const oceanGrad = ctx.createLinearGradient(-400, -200, 400, 300);
        oceanGrad.addColorStop(0, '#0c4a6e');
        oceanGrad.addColorStop(0.4, '#0369a1');
        oceanGrad.addColorStop(0.7, '#0284c7');
        oceanGrad.addColorStop(1, '#38bdf8');
        ctx.fillStyle = oceanGrad;
        ctx.beginPath();
        ctx.ellipse(0, 40, 520, 260, 0, 0, Math.PI * 2);
        ctx.fill();

        // Ocean Wave Swells & Surf Animation
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.28)';
        ctx.lineWidth = 2.5;
        for (let i = 0; i < 7; i++) {
            const waveY = -60 + i * 42 + Math.sin(now * 0.0025 + i * 1.5) * 8;
            const waveAmp = 12 + Math.sin(now * 0.003 + i) * 6;
            ctx.beginPath();
            ctx.moveTo(-360, waveY);
            ctx.bezierCurveTo(-180, waveY + waveAmp, 0, waveY - waveAmp, 200, waveY + waveAmp);
            ctx.stroke();
        }

        // 2. Coastal Shoreline & Sandy Beach Bluff
        const landGrad = ctx.createLinearGradient(-100, -260, 360, 100);
        landGrad.addColorStop(0, '#15803d'); // Green belt
        landGrad.addColorStop(0.55, '#166534');
        landGrad.addColorStop(0.85, '#d97706'); // Sandy bluff
        landGrad.addColorStop(1, '#fde68a'); // Surf sand
        ctx.fillStyle = landGrad;
        ctx.beginPath();
        ctx.moveTo(80, -240);
        ctx.bezierCurveTo(240, -180, 380, -40, 340, 120);
        ctx.bezierCurveTo(300, 220, 120, 240, -40, 200);
        ctx.bezierCurveTo(-10, 120, 20, 40, 60, -80);
        ctx.closePath();
        ctx.fill();
        ctx.strokeStyle = 'rgba(254, 243, 199, 0.4)';
        ctx.lineWidth = 3;
        ctx.stroke();

        // Coastal Shore Surf Foam
        const surfPhase = Math.sin(now * 0.004) * 6;
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.65)';
        ctx.lineWidth = 4;
        ctx.beginPath();
        ctx.moveTo(330 + surfPhase, 110);
        ctx.bezierCurveTo(290 + surfPhase, 210, 110 + surfPhase, 230, -35 + surfPhase, 190);
        ctx.stroke();

        // 3. Storm Surge Barrier Wall & Floodgates
        ctx.save();
        ctx.translate(-20, 110);
        ctx.fillStyle = '#334155';
        ctx.fillRect(-80, -14, 160, 28);
        ctx.strokeStyle = '#64748b';
        ctx.lineWidth = 2;
        ctx.strokeRect(-80, -14, 160, 28);

        // Hydraulic floodgate pylons
        for (let p = -60; p <= 60; p += 30) {
            ctx.fillStyle = '#1e293b';
            ctx.fillRect(p - 6, -22, 12, 44);
            ctx.fillStyle = '#f59e0b'; // Hazard warning chevrons
            ctx.fillRect(p - 4, -20, 8, 4);
            ctx.fillStyle = '#22c55e'; // Green operational beacon
            ctx.beginPath();
            ctx.arc(p, -24, 2.5, 0, Math.PI * 2);
            ctx.fill();
        }
        ctx.restore();

        // 4. Offshore Wind Farm (4 Turbines)
        const turbines = [
            { x: -280, y: -90, scale: 0.85, phase: 0 },
            { x: -210, y: -160, scale: 0.75, phase: 1.2 },
            { x: -160, y: -40, scale: 0.95, phase: 2.1 },
            { x: -250, y: 50, scale: 1.05, phase: 0.7 },
        ];

        turbines.forEach(tb => {
            ctx.save();
            ctx.translate(tb.x, tb.y);
            ctx.scale(tb.scale, tb.scale);

            // Underwater foundation shadow
            ctx.fillStyle = 'rgba(2, 44, 34, 0.45)';
            ctx.beginPath(); ctx.ellipse(0, 18, 14, 7, 0, 0, Math.PI * 2); ctx.fill();

            // Concrete base platform
            ctx.fillStyle = '#e2e8f0';
            ctx.beginPath(); ctx.ellipse(0, 16, 9, 4.5, 0, 0, Math.PI * 2); ctx.fill();
            ctx.fillStyle = '#eab308'; // High-visibility yellow deck
            ctx.fillRect(-6, 12, 12, 4);

            // Turbine tower mast
            ctx.fillStyle = '#f8fafc';
            ctx.beginPath();
            ctx.moveTo(-2.5, 12);
            ctx.lineTo(-1.2, -45);
            ctx.lineTo(1.2, -45);
            ctx.lineTo(2.5, 12);
            ctx.closePath();
            ctx.fill();
            ctx.strokeStyle = '#cbd5e1';
            ctx.lineWidth = 1;
            ctx.stroke();

            // Nacelle pod
            ctx.fillStyle = '#f1f5f9';
            ctx.fillRect(-5, -49, 10, 6);

            // Aeronautical safety beacon
            const flash = Math.sin(now * 0.005 + tb.phase * 3) > 0.5;
            ctx.fillStyle = flash ? '#ef4444' : '#7f1d1d';
            ctx.beginPath(); ctx.arc(0, -51, 2, 0, Math.PI * 2); ctx.fill();

            // Spinning 3-Blade Rotor
            const rot = now * 0.0035 + tb.phase;
            for (let b = 0; b < 3; b++) {
                const angle = rot + (b * Math.PI * 2) / 3;
                ctx.save();
                ctx.translate(0, -46);
                ctx.rotate(angle);
                ctx.fillStyle = '#f8fafc';
                ctx.beginPath();
                ctx.moveTo(-1.2, 0);
                ctx.lineTo(-0.6, -34);
                ctx.lineTo(0.6, -34);
                ctx.lineTo(1.2, 0);
                ctx.closePath();
                ctx.fill();
                // Red blade tips
                ctx.fillStyle = '#ef4444';
                ctx.fillRect(-0.8, -34, 1.6, 6);
                ctx.restore();
            }

            // Rotor central hub cap
            ctx.fillStyle = '#0f172a';
            ctx.beginPath(); ctx.arc(0, -46, 2.5, 0, Math.PI * 2); ctx.fill();

            ctx.restore();
        });

        // 5. Shoreline Desalination Plant Complex
        ctx.save();
        ctx.translate(140, 10);

        // Seawater Intake Pipe running into ocean
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 6;
        ctx.beginPath();
        ctx.moveTo(0, 20);
        ctx.bezierCurveTo(-40, 30, -90, 10, -130, 25);
        ctx.stroke();

        // Pulsing water flow along pipe
        const flowOff = (now * 0.05) % 20;
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 2.5;
        ctx.setLineDash([6, 14]);
        ctx.lineDashOffset = -flowOff;
        ctx.beginPath();
        ctx.moveTo(0, 20);
        ctx.bezierCurveTo(-40, 30, -90, 10, -130, 25);
        ctx.stroke();
        ctx.setLineDash([]);

        // Desalination Main Hall & Filtration Modules
        ctx.fillStyle = '#1e293b';
        ctx.fillRect(-10, -20, 60, 40);
        ctx.fillStyle = '#0284c7';
        ctx.fillRect(-6, -16, 52, 12);
        ctx.fillStyle = '#f8fafc';
        ctx.font = 'bold 8px ' + FONT_MONO;
        ctx.fillText('DESAL', 4, -7);

        // Water Storage Silos
        for (let s = 0; s < 2; s++) {
            ctx.fillStyle = '#e2e8f0';
            ctx.beginPath();
            ctx.ellipse(12 + s * 22, -32, 9, 5, 0, 0, Math.PI * 2);
            ctx.fill();
            ctx.fillRect(3 + s * 22, -32, 18, 16);
            ctx.fillStyle = '#0284c7';
            ctx.fillRect(3 + s * 22, -22, 18, 4);
        }

        // Vapor plume
        const plumeY = Math.sin(now * 0.004) * 4;
        ctx.fillStyle = 'rgba(255, 255, 255, 0.45)';
        ctx.beginPath();
        ctx.arc(44, -40 + plumeY, 7, 0, Math.PI * 2);
        ctx.arc(48, -48 + plumeY, 9, 0, Math.PI * 2);
        ctx.fill();

        ctx.restore();

        // 6. Working Harbor & Berthed Cargo Vessel
        ctx.save();
        ctx.translate(210, 110);

        // Concrete Pier
        ctx.fillStyle = '#475569';
        ctx.fillRect(-20, -10, 70, 20);
        ctx.strokeStyle = '#94a3b8';
        ctx.lineWidth = 1.5;
        ctx.strokeRect(-20, -10, 70, 20);

        // Cargo Ship Vessel in Berth
        ctx.fillStyle = '#991b1b'; // Red hull
        ctx.beginPath();
        ctx.moveTo(-10, 12);
        ctx.lineTo(60, 12);
        ctx.lineTo(75, 22);
        ctx.lineTo(-20, 22);
        ctx.closePath();
        ctx.fill();

        // Container stacks
        const containerColors = ['#0284c7', '#f59e0b', '#16a34a', '#dc2626'];
        for (let c = 0; c < 4; c++) {
            ctx.fillStyle = containerColors[c];
            ctx.fillRect(0 + c * 13, 2, 11, 10);
            ctx.strokeStyle = '#0f172a';
            ctx.lineWidth = 0.8;
            ctx.strokeRect(0 + c * 13, 2, 11, 10);
        }

        // Harbor Quay Crane
        ctx.strokeStyle = '#eab308';
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.moveTo(35, -8);
        ctx.lineTo(45, -34);
        ctx.lineTo(15, -42);
        ctx.stroke();

        ctx.restore();

        // 7. Green Belt Vertical Farming Towers
        ctx.save();
        ctx.translate(230, -120);
        for (let f = 0; f < 3; f++) {
            const fx = f * 34;
            const fy = f * 12;
            ctx.fillStyle = '#0f172a';
            ctx.fillRect(fx, fy - 36, 24, 42);
            ctx.fillStyle = '#22c55e'; // Green interior hydroponic racks
            for (let row = 0; row < 3; row++) {
                ctx.fillRect(fx + 3, fy - 32 + row * 11, 18, 6);
            }
            ctx.fillStyle = '#38bdf8'; // Solar glass angled roof
            ctx.beginPath();
            ctx.moveTo(fx, fy - 36);
            ctx.lineTo(fx + 12, fy - 46);
            ctx.lineTo(fx + 24, fy - 36);
            ctx.closePath();
            ctx.fill();
        }
        ctx.restore();

        // 8. Coastal Resilience Tactical HUD Banner
        ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 1.5;
        ctx.strokeRect(-190, 195, 380, 36);
        ctx.fillRect(-190, 195, 380, 36);

        ctx.fillStyle = '#f8fafc';
        ctx.font = 'bold 11px ' + FONT_SANS;
        ctx.textAlign = 'center';
        ctx.fillText('🌊 COASTAL RESILIENCE MATRIX · SEASCAPE & INFRASTRUCTURE', 0, 210);
        ctx.font = '10px ' + FONT_SANS;
        ctx.fillStyle = '#7dd3fc';
        ctx.fillText('⚡ 4 Offshore Turbines  ·  💧 Desal Plant Active  ·  🛡️ Surge Barrier Ready  ·  🚢 Harbor Cargo', 0, 224);

        ctx.restore();
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

        this.updateImmersiveCamera(now);

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
        if (this.perspectiveMode === 'first') ctx.rotate(-this.walker.heading);
        ctx.scale(this.camera.zoom, this.camera.zoom);

        const curWorld = (state.selectedWorld || '').toLowerCase();
        if (this.worldLens === 'city' || (curWorld.includes('city') && this.worldLens !== 'schematic' && this.worldLens !== 'realm')) {
            this.renderCityView(ctx, w, h, now);
        } else if (this.worldLens === 'realm') {
            this.renderRealmView(ctx, w, h, now);
        } else if (curWorld.includes('eco') && this.viewMode === 'cg') {
            this.renderWildernessEcosystemView(ctx, w, h, now);
        } else if (curWorld.includes('coastal') && this.viewMode === 'cg') {
            this.renderCoastalResilienceView(ctx, w, h, now);
        } else {
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
        }

        ctx.restore();

        if (this.perspectiveMode !== 'strategic') this.drawImmersiveOverlay(ctx, w, h, now);
        ctx.restore();

        // 7. Tactical Minimap Radar
        this.renderMinimap(now);

        // 8. Entity Avatar Hero Command Module
        this.renderAvatar(now);
    },

    drawImmersiveOverlay(ctx, w, h, now) {
        ctx.save();
        const first = this.perspectiveMode === 'first';
        const horizon = first ? h * 0.44 : h * 0.2;
        const shade = ctx.createLinearGradient(0, 0, 0, h);
        shade.addColorStop(0, 'rgba(3,7,14,.34)');
        shade.addColorStop(horizon / h, 'rgba(3,7,14,0)');
        shade.addColorStop(1, first ? 'rgba(3,7,14,.5)' : 'rgba(3,7,14,.22)');
        ctx.fillStyle = shade;
        ctx.fillRect(0, 0, w, h);
        if (first) {
            ctx.strokeStyle = 'rgba(66,211,234,.12)';
            ctx.lineWidth = 1;
            for (let i = 1; i <= 7; i++) {
                const y = horizon + (h - horizon) * Math.pow(i / 7, 1.7);
                ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
            }
            for (let i = -7; i <= 7; i++) {
                ctx.beginPath(); ctx.moveTo(w / 2, horizon); ctx.lineTo(w / 2 + i * w * .13, h); ctx.stroke();
            }
        } else {
            const focus = this.nodes.get(this.selectedNodeName);
            if (focus) {
                const x = w / 2 + focus.x * this.camera.zoom + this.camera.x;
                const y = h / 2 + focus.y * this.camera.zoom + this.camera.y;
                ctx.strokeStyle = 'rgba(241,185,107,.72)';
                ctx.lineWidth = 1.5;
                ctx.setLineDash([5, 5]);
                ctx.beginPath(); ctx.arc(x, y, 38 + Math.sin(now * .004) * 4, 0, Math.PI * 2); ctx.stroke();
                ctx.setLineDash([]);
            }
        }
        if (this.immersionPulse.alpha > 0) {
            ctx.globalAlpha = this.immersionPulse.alpha;
            ctx.fillStyle = this.immersionPulse.color;
            ctx.fillRect(0, 0, w, h);
            ctx.globalAlpha = 1;
            this.immersionPulse.alpha = Math.max(0, this.immersionPulse.alpha - 0.012);
        }
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

        // Environmental weather & atmosphere particle FX
        if (this.vfxEnabled && this.envWeather.length > 0) {
            const isEco = this.envWeatherType === 'ecosystem';
            const isCoast = this.envWeatherType === 'coastal';

            this.envWeather.forEach(p => {
                p.x += p.speedX;
                p.y += p.speedY;

                if (isEco) {
                    p.x += Math.sin(now * 0.002 + p.phase) * 0.45;
                    if (p.y < -h / 2 - 250) p.y = h / 2 + 250;
                }
                if (isCoast) {
                    if (p.x > w / 2 + 350) p.x = -w / 2 - 350;
                }
                if (p.x > w / 2 + 350) p.x = -w / 2 - 350;
                if (p.x < -w / 2 - 350) p.x = w / 2 + 350;
                if (p.y > h / 2 + 250) p.y = -h / 2 - 250;
                if (p.y < -h / 2 - 250) p.y = h / 2 + 250;

                const screenX = w / 2 + p.x + this.camera.x * 0.35;
                const screenY = h / 2 + p.y + this.camera.y * 0.35;

                // Pulsing glow alpha
                const pulseAlpha = isEco ? p.alpha * (0.6 + 0.4 * Math.sin(now * 0.004 + p.phase)) : p.alpha;
                const hexA = Math.floor(pulseAlpha * 255).toString(16).padStart(2, '0');

                ctx.beginPath();
                ctx.arc(screenX, screenY, p.r, 0, Math.PI * 2);
                ctx.fillStyle = `${p.color}${hexA}`;
                if (isEco && p.r > 2) {
                    ctx.shadowColor = p.color;
                    ctx.shadowBlur = 6;
                }
                ctx.fill();
                ctx.shadowBlur = 0;
            });
        }

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
        const chevronOffset = (now * 0.0007) % 0.33;

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

            // Selected node conduit telemetry link
            const isConnectedToSelected = this.selectedNodeName && (link.from === this.selectedNodeName || link.to === this.selectedNodeName);
            if (isConnectedToSelected) {
                ctx.save();
                ctx.beginPath();
                ctx.moveTo(from.x, from.y);
                ctx.quadraticCurveTo(cp.x, cp.y, to.x, to.y);
                ctx.strokeStyle = '#f1b96b';
                ctx.lineWidth = 2.2;
                ctx.shadowColor = '#f1b96b';
                ctx.shadowBlur = 8;
                ctx.stroke();
                ctx.restore();
            }

            // Animated directional flow chevrons (> > >)
            if (this.vfxEnabled) {
                for (let i = 0; i < 3; i++) {
                    const ct = (chevronOffset + i * 0.33) % 1.0;
                    if (ct > 0.16 && ct < 0.84) {
                        const cx = (1 - ct) * (1 - ct) * from.x + 2 * (1 - ct) * ct * cp.x + ct * ct * to.x;
                        const cy = (1 - ct) * (1 - ct) * from.y + 2 * (1 - ct) * ct * cp.y + ct * ct * to.y;
                        const tx = 2 * (1 - ct) * (cp.x - from.x) + 2 * ct * (to.x - cp.x);
                        const ty = 2 * (1 - ct) * (cp.y - from.y) + 2 * ct * (to.y - cp.y);
                        const angle = Math.atan2(ty, tx);

                        ctx.save();
                        ctx.translate(cx, cy);
                        ctx.rotate(angle);
                        ctx.strokeStyle = `${link.color}bb`;
                        ctx.lineWidth = 1.5;
                        ctx.beginPath();
                        ctx.moveTo(-3.5, -3);
                        ctx.lineTo(2.5, 0);
                        ctx.lineTo(-3.5, 3);
                        ctx.stroke();
                        ctx.restore();
                    }
                }
            }
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
                if (Math.random() < 0.3) {
                    this.shockwaves.push({
                        x: to.x, y: to.y, r: to.radius - 4, maxR: to.radius + 14,
                        color: p.color, alpha: 0.6, width: 1.4,
                    });
                }
            }

            const cp = this.getConduitControlPoint(from, to);
            const t = p.t;
            const px = (1 - t) * (1 - t) * from.x + 2 * (1 - t) * t * cp.x + t * t * to.x;
            const py = (1 - t) * (1 - t) * from.y + 2 * (1 - t) * t * cp.y + t * t * to.y;

            // Fading comet motion trail
            for (let s = 1; s <= 3; s++) {
                const ts = Math.max(0, t - s * 0.02);
                const sx = (1 - ts) * (1 - ts) * from.x + 2 * (1 - ts) * ts * cp.x + ts * ts * to.x;
                const sy = (1 - ts) * (1 - ts) * from.y + 2 * (1 - ts) * ts * cp.y + ts * ts * to.y;
                ctx.beginPath();
                ctx.arc(sx, sy, p.r * (1 - s * 0.22), 0, Math.PI * 2);
                ctx.fillStyle = `${p.color}${Math.floor((0.45 / s) * 255).toString(16).padStart(2, '0')}`;
                ctx.fill();
            }

            // Comet Head with bright core
            ctx.beginPath();
            ctx.arc(px, py, p.r, 0, Math.PI * 2);
            ctx.fillStyle = '#ffffff';
            ctx.shadowColor = p.color;
            ctx.shadowBlur = 10;
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
        ctx.font = '650 10.5px ' + FONT_MONO;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
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

            const tw = ctx.measureText(fl.text).width;
            const padX = 6;
            const badgeW = tw + padX * 2;
            const badgeH = 17;
            const hexAlpha = Math.floor(fl.alpha * 255).toString(16).padStart(2, '0');

            ctx.save();
            ctx.translate(fl.x, fl.y);

            // Translucent dark pill background
            ctx.fillStyle = `rgba(6,10,16,${(fl.alpha * 0.88).toFixed(2)})`;
            ctx.strokeStyle = `${fl.color}${Math.floor(fl.alpha * 180).toString(16).padStart(2, '0')}`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.roundRect(-badgeW / 2, -badgeH / 2, badgeW, badgeH, 4);
            ctx.fill();
            ctx.stroke();

            // Text
            ctx.fillStyle = `${fl.color}${hexAlpha}`;
            ctx.shadowColor = fl.color;
            ctx.shadowBlur = fl.type === 'shortage' ? 8 : 4;
            ctx.fillText(fl.text, 0, 0.5);
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
                ctx.font = '700 8px ' + FONT_MONO;
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
            ctx.font = '600 11px ' + FONT_SANS;
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
            ctx.font = '500 9px ' + FONT_MONO;
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
            case 'wildlife': {
                // Herbivore / Deer silhouette with branching antlers & breathing pulse
                const breath = Math.sin(now * 0.0035) * 0.8;
                ctx.save();
                ctx.translate(0, breath);
                // Head shape
                ctx.beginPath();
                ctx.moveTo(-4, 4);
                ctx.lineTo(0, 9);
                ctx.lineTo(4, 4);
                ctx.lineTo(3, -2);
                ctx.lineTo(-3, -2);
                ctx.closePath();
                ctx.stroke();
                // Antlers
                ctx.beginPath();
                ctx.moveTo(-2, -2); ctx.lineTo(-6, -8); ctx.lineTo(-9, -7);
                ctx.moveTo(-6, -8); ctx.lineTo(-7, -11);
                ctx.moveTo(2, -2); ctx.lineTo(6, -8); ctx.lineTo(9, -7);
                ctx.moveTo(6, -8); ctx.lineTo(7, -11);
                ctx.stroke();
                // Eye dot
                ctx.beginPath();
                ctx.arc(0, 2, 1.2, 0, Math.PI * 2);
                ctx.fill();
                ctx.restore();
                break;
            }
            case 'predator': {
                // Apex Predator / Wolf head silhouette with alert ears & glowing eye
                const pulse = Math.sin(now * 0.004) * 0.9;
                ctx.save();
                ctx.translate(0, pulse);
                // Wolf head outline
                ctx.beginPath();
                ctx.moveTo(0, 9);
                ctx.lineTo(-4, 3);
                ctx.lineTo(-7, -2);
                ctx.lineTo(-6, -10); // left ear tip
                ctx.lineTo(-2, -5);  // head top
                ctx.lineTo(2, -5);
                ctx.lineTo(6, -10);  // right ear tip
                ctx.lineTo(7, -2);
                ctx.lineTo(4, 3);
                ctx.closePath();
                ctx.stroke();
                // Inner ears
                ctx.beginPath();
                ctx.moveTo(-5, -3); ctx.lineTo(-4, -7);
                ctx.moveTo(5, -3); ctx.lineTo(4, -7);
                ctx.stroke();
                // Ruby glowing eyes
                ctx.fillStyle = '#ff4d6d';
                ctx.beginPath();
                ctx.arc(-2, -0.5, 1.2, 0, Math.PI * 2);
                ctx.arc(2, -0.5, 1.2, 0, Math.PI * 2);
                ctx.fill();
                ctx.restore();
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

    panToMinimapCoord(mx, my) {
        if (!this.minimapBounds) return;
        const { s, midX, midY } = this.minimapBounds;
        const wx = (mx - 65) / s + midX;
        const wy = (my - 37.5) / s + midY;
        this.camera.targetX = -wx * this.camera.zoom;
        this.camera.targetY = -wy * this.camera.zoom;
        this.camera.hasInteracted = true;
    },

    renderMinimap(now) {
        if (!this.minimapCanvas || !this.minimapCtx) return;
        const ctx = this.minimapCtx;
        const w = 130, h = 75;
        ctx.clearRect(0, 0, w, h);

        const count = this.nodes.size;
        if (!count) return;

        let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
        this.nodes.forEach(node => {
            minX = Math.min(minX, node.x);
            maxX = Math.max(maxX, node.x);
            minY = Math.min(minY, node.y);
            maxY = Math.max(maxY, node.y);
        });

        const pad = 70;
        minX -= pad; maxX += pad; minY -= pad; maxY += pad;
        const boxW = Math.max(160, maxX - minX);
        const boxH = Math.max(90, maxY - minY);
        const s = Math.min((w - 14) / boxW, (h - 14) / boxH);
        const midX = (minX + maxX) / 2;
        const midY = (minY + maxY) / 2;

        this.minimapBounds = { s, midX, midY };

        const toMx = wx => w / 2 + (wx - midX) * s;
        const toMy = wy => h / 2 + (wy - midY) * s;

        // Draw radar sweep line
        const sweepLen = 50;
        const cx = w / 2, cy = h / 2;
        const sweepGrad = ctx.createLinearGradient(cx, cy, cx + Math.cos(this.radarAngle) * sweepLen, cy + Math.sin(this.radarAngle) * sweepLen);
        sweepGrad.addColorStop(0, 'rgba(66,211,234,0.18)');
        sweepGrad.addColorStop(1, 'transparent');
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.arc(cx, cy, sweepLen, this.radarAngle - 0.45, this.radarAngle);
        ctx.closePath();
        ctx.fillStyle = sweepGrad;
        ctx.fill();

        // Subtle radar range circles
        ctx.strokeStyle = 'rgba(66,211,234,0.12)';
        ctx.lineWidth = 1;
        [16, 32, 48].forEach(r => {
            ctx.beginPath();
            ctx.arc(cx, cy, r, 0, Math.PI * 2);
            ctx.stroke();
        });

        // Draw links
        ctx.strokeStyle = 'rgba(103,169,255,0.24)';
        ctx.lineWidth = 1;
        this.links.forEach(link => {
            const from = this.nodes.get(link.from);
            const to = this.nodes.get(link.to);
            if (!from || !to) return;
            ctx.beginPath();
            ctx.moveTo(toMx(from.x), toMy(from.y));
            ctx.lineTo(toMx(to.x), toMy(to.y));
            ctx.stroke();
        });

        // Draw nodes
        this.nodes.forEach(node => {
            const mx = toMx(node.x);
            const my = toMy(node.y);
            const isSelected = this.selectedNodeName === node.name;

            if (isSelected) {
                ctx.beginPath();
                ctx.arc(mx, my, 4.5, 0, Math.PI * 2);
                ctx.fillStyle = 'rgba(241,185,107,0.3)';
                ctx.fill();
                ctx.strokeStyle = '#f1b96b';
                ctx.lineWidth = 1.2;
                ctx.stroke();
            }

            ctx.beginPath();
            ctx.arc(mx, my, isSelected ? 2.5 : 1.8, 0, Math.PI * 2);
            ctx.fillStyle = isSelected ? '#ffffff' : node.theme.color;
            ctx.fill();
        });

        // Camera Frustum Box
        if (this.canvas) {
            const cw = this.canvas.clientWidth || 800;
            const ch = this.canvas.clientHeight || 420;
            const camWx1 = (-cw / 2 - this.camera.x) / this.camera.zoom;
            const camWy1 = (-ch / 2 - this.camera.y) / this.camera.zoom;
            const camWx2 = (cw / 2 - this.camera.x) / this.camera.zoom;
            const camWy2 = (ch / 2 - this.camera.y) / this.camera.zoom;

            const rx1 = Math.max(1, toMx(camWx1));
            const ry1 = Math.max(1, toMy(camWy1));
            const rx2 = Math.min(w - 1, toMx(camWx2));
            const ry2 = Math.min(h - 1, toMy(camWy2));

            const rw = Math.max(6, rx2 - rx1);
            const rh = Math.max(4, ry2 - ry1);

            ctx.fillStyle = 'rgba(66,211,234,0.12)';
            ctx.fillRect(rx1, ry1, rw, rh);
            ctx.strokeStyle = 'rgba(66,211,234,0.85)';
            ctx.lineWidth = 1.2;
            ctx.strokeRect(rx1, ry1, rw, rh);
        }
    },

    renderAvatar(now) {
        if (!this.avatarCanvas || !this.avatarCtx) return;
        const ctx = this.avatarCtx;
        const w = 56, h = 56;
        ctx.clearRect(0, 0, w, h);

        const node = this.nodes.get(this.selectedNodeName) || (this.nodes.size > 0 ? this.nodes.values().next().value : null);
        if (!node) {
            ctx.fillStyle = '#64748b';
            ctx.font = '8px ' + FONT_MONO;
            ctx.textAlign = 'center';
            ctx.fillText('SELECT ENTITY', w / 2, h / 2 + 3);
            return;
        }

        const cx = w / 2, cy = h / 2;
        const themeColor = node.theme.color;

        ctx.save();
        ctx.translate(cx, cy);

        // Subtle background radial glow
        const bgGlow = ctx.createRadialGradient(0, 0, 4, 0, 0, 26);
        bgGlow.addColorStop(0, `${themeColor}33`);
        bgGlow.addColorStop(1, 'transparent');
        ctx.beginPath();
        ctx.arc(0, 0, 26, 0, Math.PI * 2);
        ctx.fillStyle = bgGlow;
        ctx.fill();

        // Outer rotating scanner ring with tick marks
        const rot = (now * 0.001) % (Math.PI * 2);
        ctx.save();
        ctx.rotate(rot);
        ctx.strokeStyle = `${themeColor}66`;
        ctx.lineWidth = 1.2;
        ctx.setLineDash([4, 6]);
        ctx.beginPath();
        ctx.arc(0, 0, 24, 0, Math.PI * 2);
        ctx.stroke();
        ctx.restore();

        // Inner reticle frame
        ctx.strokeStyle = `${themeColor}aa`;
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.arc(0, 0, 19, 0, Math.PI * 2);
        ctx.stroke();

        // 4 Corner HUD brackets
        const bSize = 4, bDist = 21;
        ctx.strokeStyle = 'rgba(255,255,255,0.7)';
        ctx.lineWidth = 1;
        [[-bDist, -bDist, 1, 1], [bDist, -bDist, -1, 1], [-bDist, bDist, 1, -1], [bDist, bDist, -1, -1]].forEach(([bx, by, dx, dy]) => {
            ctx.beginPath();
            ctx.moveTo(bx + dx * bSize, by);
            ctx.lineTo(bx, by);
            ctx.lineTo(bx, by + dy * bSize);
            ctx.stroke();
        });

        // Procedural animated archetype icon in center
        this.drawArchetypeGraphic(ctx, node.archetype, themeColor, now, node.capacity);

        // Overdrive sparks if capacity > 1.0
        if (node.capacity > 1.0) {
            this.drawOverdriveSparks(ctx, 15, now);
        }

        ctx.restore();
    },
};

// =========================================================================
// EVOLUTION TREES, OVERSEER POWERS, PROGRESSION & TROPHIES ENGINE
// =========================================================================

let activeTreeBranch = 'all';

function formatPerks(effects) {
    if (!effects) return '';
    const perks = [];
    if (effects.wellbeing_bonus) perks.push(`+${effects.wellbeing_bonus} Wellbeing`);
    if (effects.housing_multiplier && effects.housing_multiplier !== 1) {
        perks.push(`+${Math.round((effects.housing_multiplier - 1) * 100)}% Housing`);
    }
    if (effects.jobs_multiplier && effects.jobs_multiplier !== 1) {
        const sign = effects.jobs_multiplier > 1 ? '+' : '';
        perks.push(`${sign}${Math.round((effects.jobs_multiplier - 1) * 100)}% Jobs`);
    }
    if (effects.resource_multipliers) {
        for (const [res, mult] of Object.entries(effects.resource_multipliers)) {
            perks.push(`+${Math.round((mult - 1) * 100)}% ${prettyName(res)}`);
        }
    }
    if (effects.building_multipliers) {
        for (const [bld, mult] of Object.entries(effects.building_multipliers)) {
            perks.push(`+${Math.round((mult - 1) * 100)}% ${prettyName(bld)}`);
        }
    }
    return perks.join(' · ');
}

function createTechnologyCard(technology) {
    const node = document.createElement('article');
    const branchClass = `branch-${technology.branch || 'technology'}`;
    const statusClass = technology.researched ? 'researched' : technology.available ? 'available' : technology.excluded ? 'excluded' : 'locked';
    node.className = `technology-node ${branchClass} ${statusClass}`;

    const head = document.createElement('div');
    head.className = 'technology-node-header';
    const title = document.createElement('strong'); title.textContent = technology.name;
    const branch = document.createElement('small');
    branch.textContent = technology.branch ? technology.branch.toUpperCase() : 'TECH';
    head.append(title, branch);

    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = technology.researched ? 'Researched' : technology.excluded ? 'Excluded' : 'Research';
    button.disabled = technology.researched || !technology.available || !technology.affordable || state.play.requestInFlight || state.play.data?.completed || technology.excluded;
    button.addEventListener('click', () => {
        WorldForgeCG.audio?.playUnlock?.();
        spawnFloatingText(`🔬 UNLOCKED: ${technology.name}`, null, null, 'fx-success');
        researchCityTechnology(technology.id);
    });

    const copy = document.createElement('p'); copy.textContent = technology.description;

    const cost = document.createElement('span'); cost.className = 'tech-cost';
    cost.textContent = technology.researched ? 'Integrated into the city' : `${resourceList(technology.cost)}${technology.prerequisites.length ? ` · requires ${technology.prerequisites.map(prettyName).join(', ')}` : ''}`;

    const perkText = formatPerks(technology.effects);
    const perk = document.createElement('span'); perk.className = 'tech-perk';
    perk.textContent = perkText ? `★ ${perkText}` : '';

    node.append(head, button, copy, cost);
    if (perkText) node.append(perk);
    return node;
}

function spawnFloatingText(text, x, y, type = 'fx-surge') {
    const layer = el('floating-fx-layer');
    if (!layer) return;
    const node = document.createElement('div');
    node.className = `floating-text ${type}`;
    node.textContent = text;
    if (x == null || y == null) {
        const rect = el('play-network-container')?.getBoundingClientRect() || document.body.getBoundingClientRect();
        x = rect.left + rect.width / 2 + (Math.random() - 0.5) * 200;
        y = rect.top + rect.height / 2 + (Math.random() - 0.5) * 100;
    }
    node.style.left = `${Math.max(10, Math.min(window.innerWidth - 180, x))}px`;
    node.style.top = `${Math.max(40, y)}px`;
    layer.appendChild(node);
    setTimeout(() => node.remove(), 1400);
}

function setupTreeTabs() {
    const tabContainers = [el('tree-branch-tabs'), el('modal-branch-tabs')];
    tabContainers.forEach(container => {
        if (!container) return;
        container.addEventListener('click', (e) => {
            const btn = e.target.closest('.tree-tab');
            if (!btn) return;
            activeTreeBranch = btn.dataset.branch || 'all';
            tabContainers.forEach(c => {
                if (!c) return;
                c.querySelectorAll('.tree-tab').forEach(t => {
                    t.classList.toggle('active', (t.dataset.branch || 'all') === activeTreeBranch);
                });
            });
            if (state.play?.data?.city) {
                renderCityLayer(state.play.data.city);
            }
        });
    });
}

function setupCommanderPowers() {
    el('pwr-overdrive')?.addEventListener('click', () => {
        WorldForgeCG.audio?.playOverdrive?.();
        spawnFloatingText('⚡ 150% GRID OVERDRIVE ACTIVE!', null, null, 'fx-surge');
        const slider = el('capacity-slider');
        if (slider) {
            slider.value = 150;
            updateCapacityLabel();
            syncPresetHighlight(150);
            applyCapacityDecision();
        }
    });

    el('pwr-festival')?.addEventListener('click', () => {
        WorldForgeCG.audio?.playLevelUp?.();
        spawnFloatingText('🎉 GRAND CIVIC FESTIVAL (+10 WELLBEING)!', null, null, 'fx-culture');
        showToast('Festival Celebrated', 'A grand festival sweeps the districts, elevating civic unity and wellbeing!');
        if (state.play?.data?.city) {
            state.play.data.city.wellbeing = (state.play.data.city.wellbeing || 0) + 10.0;
            renderCityLayer(state.play.data.city);
        }
    });

    el('pwr-eureka')?.addEventListener('click', () => {
        WorldForgeCG.audio?.playUnlock?.();
        spawnFloatingText('🧪 EUREKA BREAKTHROUGH (+40 RESEARCH)!', null, null, 'fx-success');
        showToast('Eureka Surge', 'Municipal laboratories achieved an unexpected research breakthrough!');
        const available = state.play?.data?.city?.technologies?.find(t => !t.researched && t.available);
        if (available) {
            showToast('Breakthrough ready', `Research ${available.name} in the Evolution Tree.`);
        }
    });

    el('pwr-relief')?.addEventListener('click', () => {
        WorldForgeCG.audio?.playConstruction?.();
        spawnFloatingText('📦 RELIEF AIRDROP: +100 WATER, +100 FOOD!', null, null, 'fx-warning');
        showToast('Emergency Relief Dispatched', 'Emergency provisions air-dropped across distressed sectors.');
    });

    el('pwr-crisis')?.addEventListener('click', () => {
        WorldForgeCG.audio?.playChaos?.();
        spawnFloatingText('🌪️ CHAOS SURGE: SOLAR STORM DETECTED!', null, null, 'fx-danger');
        showToast('Stochastic Crisis Triggered', 'A fierce solar storm tests district grids! Monitor shortages.', 'error');
        const slider = el('capacity-slider');
        if (slider && Number(slider.value) > 80) {
            slider.value = 80;
            updateCapacityLabel();
            syncPresetHighlight(80);
            applyCapacityDecision();
        }
    });
}

function openTreeModal() {
    const dialog = el('tree-modal');
    if (!dialog) return;
    if (state.play?.data?.city) {
        renderCityLayer(state.play.data.city);
    } else {
        showToast('Evolution Constellation', 'Open Play World in Micro-City to interact with the full live evolution trees.');
    }
    dialog.showModal();
}

function closeTreeModal() {
    el('tree-modal')?.close();
}

function openTrophiesModal() {
    renderTrophiesGrid();
    el('trophies-dialog')?.showModal();
}

function closeTrophiesModal() {
    el('trophies-dialog')?.close();
}

function closeLevelUpBanner() {
    el('level-up-banner')?.classList.add('hidden');
}

let currentCivilizationLevel = 1;
const ERA_TIERS = [
    { level: 1, name: "Frontier Outpost", badge: "ERA I", minScore: 0, maxScore: 300, desc: "A fledgling outpost establishing basic power, water, and food networks." },
    { level: 2, name: "Thriving Settlement", badge: "ERA II", minScore: 300, maxScore: 800, desc: "Dense neighborhoods with civic institutions and expanding maker industry." },
    { level: 3, name: "Industrial Heartland", badge: "ERA III", minScore: 800, maxScore: 1800, desc: "High-throughput manufacturing, geothermal energy, and active cultural forums." },
    { level: 4, name: "Cybernetic Metropolis", badge: "ERA IV", minScore: 1800, maxScore: 3500, desc: "Autonomous distribution, clean fusion power, and deep quantum telemetry." },
    { level: 5, name: "Planetary Arcology", badge: "ERA V", minScore: 3500, maxScore: 999999, desc: "Post-scarcity prosperity and harmonious transcendence across human and natural systems." },
];

function updateCivilizationProgression(city) {
    if (!city) return;
    const researchedCount = (city.technologies || []).filter(t => t.researched).length;
    const builtCount = (city.buildings || []).reduce((acc, b) => acc + (b.count || 0), 0);
    const score = (city.population * 2) + (city.housing * 1) + (city.wellbeing * 20) + (researchedCount * 120) + (builtCount * 60);

    let activeTier = ERA_TIERS[0];
    for (let i = ERA_TIERS.length - 1; i >= 0; i--) {
        if (score >= ERA_TIERS[i].minScore) {
            activeTier = ERA_TIERS[i];
            break;
        }
    }

    const badge = el('era-badge');
    const name = el('era-name');
    const tag = el('era-level-tag');
    const xpBar = el('era-xp-bar');
    if (badge) badge.textContent = activeTier.badge;
    if (name) name.textContent = activeTier.name;
    if (tag) tag.textContent = `LVL ${activeTier.level}`;
    if (xpBar) {
        const range = Math.max(1, activeTier.maxScore - activeTier.minScore);
        const progress = Math.min(100, Math.max(5, ((score - activeTier.minScore) / range) * 100));
        xpBar.style.width = `${progress}%`;
    }

    if (activeTier.level > currentCivilizationLevel) {
        currentCivilizationLevel = activeTier.level;
        WorldForgeCG.audio?.playLevelUp?.();
        spawnFloatingText(`✦ ERA ADVANCED: ${activeTier.name.toUpperCase()}!`, null, null, 'fx-surge');
        const banner = el('level-up-banner');
        if (banner) {
            el('level-up-title').textContent = activeTier.name;
            el('level-up-copy').textContent = activeTier.desc;
            banner.classList.remove('hidden');
        }
    }
}

const TROPHIES_CATALOG = [
    { id: 'FIRST_SPARK', title: 'First Spark of Progress', icon: '🔬', desc: 'Research your first breakthrough in any development tree.' },
    { id: 'EXPANSIONIST', title: 'Master Architect', icon: '🏗️', desc: 'Construct at least 4 district buildings to anchor city infrastructure.' },
    { id: 'CULTURAL_FLOURISHING', title: 'Cultural Renaissance', icon: '🎭', desc: 'Elevate city wellbeing above 25.0 points.' },
    { id: 'CLEAN_ENERGY', title: 'Clean Power Hegemony', icon: '☀️', desc: 'Research Solar Weave Photovoltaics or Compact Fusion Core.' },
    { id: 'POST_SCARCITY', title: 'Post-Scarcity Horizon', icon: '🌐', desc: 'Adopt the Post-Scarcity Dividend economy tree capstone.' },
    { id: 'TRANSCENDENCE', title: 'Transcendence Arcology', icon: '👑', desc: 'Unlock the Grand Singularity synthesis wonder.' },
    { id: 'DEFENDER', title: 'Bastion of Peace', icon: '🛡️', desc: 'Secure diplomatic harmony across regional factions or fortify defenses to 80+ security.' },
    { id: 'MERCHANT_PRINCE', title: 'Trade Route Sovereign', icon: '🚀', desc: 'Deploy automated sky-drone logistics and amass over 400 trade reserves or 8 district buildings.' },
    { id: 'RESONANCE', title: 'Zenith Architecture', icon: '✨', desc: 'Elevate any municipal building to Level 3 Zenith Architecture.' },
    { id: 'MEGACITY', title: 'Metropolis of Tomorrow', icon: '🌆', desc: 'Surpass 1,000 citizens or construct 10 thriving district structures.' },
];
let unlockedTrophies = new Set(JSON.parse(localStorage.getItem('wf_trophies') || '[]'));

function checkTrophies(city) {
    if (!city) return;
    const researched = new Set((city.technologies || []).filter(t => t.researched).map(t => t.id));
    const builtCount = (city.buildings || []).reduce((acc, b) => acc + (b.count || 0), 0);

    function award(id) {
        if (!unlockedTrophies.has(id)) {
            unlockedTrophies.add(id);
            localStorage.setItem('wf_trophies', JSON.stringify([...unlockedTrophies]));
            const trophy = TROPHIES_CATALOG.find(t => t.id === id);
            if (trophy) {
                showToast(`🏆 Trophy Unlocked: ${trophy.title}`, trophy.desc);
                spawnFloatingText(`🏆 TROPHY: ${trophy.title}`, null, null, 'fx-culture');
                WorldForgeCG.audio?.playTrophy?.();
            }
        }
    }

    if (researched.size >= 1) award('FIRST_SPARK');
    if (builtCount >= 4) award('EXPANSIONIST');
    if (city.wellbeing >= 25.0) award('CULTURAL_FLOURISHING');
    if (researched.has('solar-weave') || researched.has('fusion-core')) award('CLEAN_ENERGY');
    if (researched.has('post-scarcity-commons')) award('POST_SCARCITY');
    if (researched.has('arcology-singularity')) award('TRANSCENDENCE');
    if (city.geopolitics?.some?.(f => f.relation >= 80) || (city.security || 0) >= 80 || city.geopolitics?.defensePosture === 'fortified') award('DEFENDER');
    if ((city.treasury || 0) >= 400 || builtCount >= 8) award('MERCHANT_PRINCE');
    if ((city.buildings || []).some(b => (b.level || 1) >= 3)) award('RESONANCE');
    if ((city.population || 0) >= 1000 || builtCount >= 10) award('MEGACITY');

    const badge = el('achieve-unlocked-count');
    if (badge) badge.textContent = `${unlockedTrophies.size}/${TROPHIES_CATALOG.length}`;
}

function renderTrophiesGrid() {
    const grid = el('trophies-grid');
    if (!grid) return;
    grid.replaceChildren(...TROPHIES_CATALOG.map(t => {
        const unlocked = unlockedTrophies.has(t.id);
        const card = document.createElement('div');
        card.className = `trophy-card ${unlocked ? 'unlocked' : 'locked'}`;
        card.innerHTML = `
            <div class="trophy-icon-wrap">${t.icon}</div>
            <div class="trophy-meta">
                <strong>${t.title}</strong>
                <p>${t.desc}</p>
                <span class="trophy-date">${unlocked ? '✦ Unlocked' : 'Locked'}</span>
            </div>
        `;
        return card;
    }));
}

initialize();
