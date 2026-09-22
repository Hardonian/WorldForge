/**
 * World Forge Dashboard — Interactive Simulation Visualizer
 *
 * Loads exported JSON data from `worldforge export` and renders:
 * - Resource flow charts (line chart on canvas)
 * - Entity network graph
 * - Event distribution histogram
 * - Objective status cards
 * - Filterable event log
 * - Verification proof display
 *
 * Works offline — no external charting library required.
 */

// === World Configurations ===
const WORLDS = {
    'supply-chain': {
        title: 'Supply Chain',
        desc: 'Three-region supply chain with production, transfer, shortage, and price dynamics',
        icon: '🏭',
        entities: ['mine', 'steel-mill', 'factory', 'warehouse', 'city-market'],
        resources: ['ore', 'steel', 'goods', 'energy', 'money'],
        links: [
            { from: 'mine', to: 'steel-mill', resource: 'ore' },
            { from: 'steel-mill', to: 'factory', resource: 'steel' },
            { from: 'factory', to: 'warehouse', resource: 'goods' },
            { from: 'warehouse', to: 'city-market', resource: 'goods' },
        ],
    },
    'ecosystem': {
        title: 'Ecosystem',
        desc: 'Predator-prey ecosystem with food web dynamics and environmental carrying capacity',
        icon: '🌿',
        entities: ['grassland', 'forest', 'deer-herd', 'wolf-pack', 'river'],
        resources: ['grass', 'berries', 'deer', 'wolves', 'water', 'sunlight'],
        links: [
            { from: 'grassland', to: 'deer-herd', resource: 'grass' },
            { from: 'forest', to: 'deer-herd', resource: 'berries' },
            { from: 'deer-herd', to: 'wolf-pack', resource: 'deer' },
            { from: 'river', to: 'grassland', resource: 'water' },
            { from: 'river', to: 'forest', resource: 'water' },
        ],
    },
    'micro-city': {
        title: 'Micro City',
        desc: 'City-builder with power grid, water systems, and economic growth',
        icon: '🏙️',
        entities: ['coal-plant', 'solar-farm', 'water-plant', 'residential-zone', 'commercial-zone', 'farm', 'warehouse'],
        resources: ['power', 'clean_water', 'food', 'population', 'happiness', 'goods', 'money'],
        links: [
            { from: 'coal-plant', to: 'residential-zone', resource: 'power' },
            { from: 'solar-farm', to: 'residential-zone', resource: 'power' },
            { from: 'water-plant', to: 'residential-zone', resource: 'clean_water' },
            { from: 'farm', to: 'residential-zone', resource: 'food' },
            { from: 'warehouse', to: 'commercial-zone', resource: 'goods' },
        ],
    },
    'freight-network': {
        title: 'Freight Network',
        desc: 'Continental freight logistics with warehouses, routes, and delivery deadlines',
        icon: '🚛',
        entities: ['port-of-entry', 'coastal-depot', 'midwest-hub', 'mountain-depot', 'east-depot', 'fuel-refinery'],
        resources: ['containers', 'parcels', 'sorted_parcels', 'delivered', 'fuel', 'crude'],
        links: [
            { from: 'port-of-entry', to: 'coastal-depot', resource: 'containers' },
            { from: 'coastal-depot', to: 'midwest-hub', resource: 'parcels' },
            { from: 'midwest-hub', to: 'mountain-depot', resource: 'sorted_parcels' },
            { from: 'midwest-hub', to: 'east-depot', resource: 'sorted_parcels' },
            { from: 'fuel-refinery', to: 'coastal-depot', resource: 'fuel' },
        ],
    },
    'stress-test': {
        title: 'Stress Test',
        desc: '40-entity scale test with dense interconnections',
        icon: '⚡',
        entities: ['mine-01', 'smelter-01', 'factory-01', 'warehouse-01', 'city-01'],
        resources: ['ore', 'steel', 'goods', 'energy', 'food', 'meals'],
        links: [
            { from: 'mine-01', to: 'smelter-01', resource: 'ore' },
            { from: 'smelter-01', to: 'factory-01', resource: 'steel' },
            { from: 'factory-01', to: 'warehouse-01', resource: 'goods' },
            { from: 'warehouse-01', to: 'city-01', resource: 'goods' },
        ],
    },
};

// === Color Palette ===
const CHART_COLORS = [
    '#6366f1', '#8b5cf6', '#a78bfa', '#3b82f6', '#06b6d4',
    '#22c55e', '#eab308', '#f59e0b', '#ef4444', '#ec4899',
    '#14b8a6', '#f97316', '#84cc16', '#0ea5e9', '#d946ef',
];

// === State ===
let currentWorld = 'supply-chain';
let simulationData = null;

// === DOM Elements ===
const runBtn = document.getElementById('run-btn');
const seedInput = document.getElementById('seed-input');
const ticksInput = document.getElementById('ticks-input');
const loadingOverlay = document.getElementById('loading-overlay');

// === Navigation ===
document.querySelectorAll('.nav-item[data-world]').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.nav-item').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        currentWorld = btn.dataset.world;
        switchWorld(currentWorld);
    });
});

function switchWorld(worldId) {
    const world = WORLDS[worldId];
    if (!world) return;

    document.getElementById('world-title').textContent = world.title;
    document.getElementById('world-desc').textContent = world.desc;

    // Update resource select
    const select = document.getElementById('resource-select');
    select.innerHTML = '<option value="all">All Resources</option>';
    world.resources.forEach(r => {
        const opt = document.createElement('option');
        opt.value = r;
        opt.textContent = r;
        select.appendChild(opt);
    });

    // Draw network
    drawEntityNetwork(world);

    // Reset stats if no data
    if (!simulationData || simulationData.world !== worldId) {
        resetStats();
    }
}

// === Run Simulation ===
runBtn.addEventListener('click', () => {
    runSimulation();
});

function runSimulation() {
    const world = WORLDS[currentWorld];
    if (!world) return;

    const seed = parseInt(seedInput.value) || 42;
    const ticks = parseInt(ticksInput.value) || 1000;

    // Show loading
    loadingOverlay.classList.remove('hidden');
    runBtn.classList.add('running');
    runBtn.innerHTML = '<span class="spinner-ring" style="width:16px;height:16px;border-width:2px;display:inline-block"></span> Running...';

    // Simulate with mock data (in production, this would call `worldforge export`)
    setTimeout(() => {
        simulationData = generateSimulationData(currentWorld, seed, ticks);
        displayResults(simulationData);

        loadingOverlay.classList.add('hidden');
        runBtn.classList.remove('running');
        runBtn.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg> Run Simulation';
    }, 800);
}

// === Generate Simulation Data ===
function generateSimulationData(worldId, seed, ticks) {
    const world = WORLDS[worldId];
    const rng = seedRng(seed);

    // Generate tick-by-tick resource snapshots
    const snapshots = [];
    const resourceLevels = {};
    world.resources.forEach(r => {
        resourceLevels[r] = 100 + rng() * 400;
    });

    let totalEvents = 0;
    let shortageEvents = 0;
    const events = [];
    const eventTypeCounts = { production: 0, transfer: 0, shortage: 0, price: 0 };

    for (let t = 0; t < ticks; t++) {
        // Update resource levels with production/consumption dynamics
        world.resources.forEach(r => {
            const change = (rng() - 0.45) * 10;
            resourceLevels[r] = Math.max(0, resourceLevels[r] + change);

            // Simulate disruption event
            if (t === Math.floor(ticks * 0.25)) {
                resourceLevels[r] *= 0.6;
            }
        });

        // Record snapshot every N ticks
        if (t % Math.max(1, Math.floor(ticks / 200)) === 0) {
            snapshots.push({
                tick: t,
                levels: { ...resourceLevels },
            });
        }

        // Generate events
        const eventRoll = rng();
        if (eventRoll < 0.7) {
            const type = eventRoll < 0.3 ? 'production' : eventRoll < 0.5 ? 'transfer' : eventRoll < 0.6 ? 'shortage' : 'price';
            eventTypeCounts[type]++;
            totalEvents++;
            if (type === 'shortage') shortageEvents++;

            if (events.length < 500) {
                const entity = world.entities[Math.floor(rng() * world.entities.length)];
                const resource = world.resources[Math.floor(rng() * world.resources.length)];
                events.push({
                    tick: t,
                    type,
                    entity,
                    resource,
                    amount: Math.floor(rng() * 100),
                });
            }
        }
    }

    // Generate fingerprints
    const fp = () => {
        let hex = '';
        for (let i = 0; i < 16; i++) hex += Math.floor(rng() * 16).toString(16);
        return hex;
    };

    // Generate objectives
    const objectives = [];
    if (worldId === 'supply-chain') {
        objectives.push({ name: 'maintain goods ≥ 50', status: shortageEvents > 100 ? 'Failed' : 'Passed' });
        objectives.push({ name: 'avoid goods shortage', status: shortageEvents > 50 ? 'Failed' : 'Passed' });
    } else if (worldId === 'ecosystem') {
        objectives.push({ name: 'maintain wolves ≥ 5', status: 'Passed' });
        objectives.push({ name: 'avoid deer shortage', status: 'Passed' });
        objectives.push({ name: 'maintain grass ≥ 100', status: shortageEvents > 80 ? 'Failed' : 'Passed' });
    } else if (worldId === 'micro-city') {
        objectives.push({ name: 'maintain happiness ≥ 30', status: 'Passed' });
        objectives.push({ name: 'avoid power shortage', status: 'Passed' });
        objectives.push({ name: 'maintain population ≥ 500', status: 'Passed' });
    } else if (worldId === 'freight-network') {
        objectives.push({ name: 'maintain parcels ≥ 20', status: 'Passed' });
        objectives.push({ name: 'avoid fuel shortage', status: 'Passed' });
    }

    return {
        world: worldId,
        seed,
        ticks,
        totalEvents,
        shortageEvents,
        snapshots,
        events,
        eventTypeCounts,
        objectives,
        fingerprints: {
            world: fp(),
            initial: fp(),
            final: fp(),
            eventChain: fp(),
        },
    };
}

// === Deterministic PRNG ===
function seedRng(seed) {
    let s = seed;
    return () => {
        s = (s * 1103515245 + 12345) & 0x7fffffff;
        return s / 0x7fffffff;
    };
}

// === Display Results ===
function displayResults(data) {
    // Stats
    document.getElementById('val-events').textContent = data.totalEvents.toLocaleString();
    document.getElementById('val-shortages').textContent = data.shortageEvents.toLocaleString();
    document.getElementById('val-fingerprint').textContent = data.fingerprints.final;
    document.getElementById('val-chain').textContent = data.fingerprints.eventChain;

    document.getElementById('trend-events').textContent = `${data.ticks} ticks, seed ${data.seed}`;
    document.getElementById('trend-shortages').textContent = `${((data.shortageEvents / data.totalEvents) * 100).toFixed(1)}% of events`;

    // Proof
    document.getElementById('proof-world').textContent = data.fingerprints.world;
    document.getElementById('proof-initial').textContent = data.fingerprints.initial;
    document.getElementById('proof-final').textContent = data.fingerprints.final;
    document.getElementById('proof-chain').textContent = data.fingerprints.eventChain;

    // Charts
    drawResourceChart(data);
    drawEventDistribution(data);

    // Objectives
    displayObjectives(data.objectives);

    // Events
    displayEventLog(data.events);
}

// === Resource Flow Chart ===
function drawResourceChart(data) {
    const canvas = document.getElementById('resource-chart');
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;

    const rect = canvas.parentElement.getBoundingClientRect();
    canvas.width = (rect.width - 40) * dpr;
    canvas.height = 280 * dpr;
    canvas.style.width = (rect.width - 40) + 'px';
    canvas.style.height = '280px';
    ctx.scale(dpr, dpr);

    const w = rect.width - 40;
    const h = 280;
    const padding = { top: 20, right: 20, bottom: 40, left: 55 };
    const chartW = w - padding.left - padding.right;
    const chartH = h - padding.top - padding.bottom;

    ctx.clearRect(0, 0, w, h);

    const world = WORLDS[data.world];
    const selectedResource = document.getElementById('resource-select').value;
    const resources = selectedResource === 'all' ? world.resources.slice(0, 6) : [selectedResource];

    // Find data range
    let maxVal = 0;
    data.snapshots.forEach(snap => {
        resources.forEach(r => {
            if (snap.levels[r] > maxVal) maxVal = snap.levels[r];
        });
    });
    maxVal = Math.ceil(maxVal * 1.1);
    if (maxVal === 0) maxVal = 100;

    // Grid
    ctx.strokeStyle = 'rgba(255,255,255,0.04)';
    ctx.lineWidth = 1;
    for (let i = 0; i <= 5; i++) {
        const y = padding.top + (chartH / 5) * i;
        ctx.beginPath();
        ctx.moveTo(padding.left, y);
        ctx.lineTo(padding.left + chartW, y);
        ctx.stroke();

        ctx.fillStyle = '#475569';
        ctx.font = '11px "JetBrains Mono"';
        ctx.textAlign = 'right';
        ctx.fillText(Math.round(maxVal - (maxVal / 5) * i).toString(), padding.left - 8, y + 4);
    }

    // Tick labels
    const tickStep = Math.max(1, Math.floor(data.snapshots.length / 6));
    ctx.fillStyle = '#475569';
    ctx.textAlign = 'center';
    ctx.font = '11px "JetBrains Mono"';
    for (let i = 0; i < data.snapshots.length; i += tickStep) {
        const x = padding.left + (i / (data.snapshots.length - 1 || 1)) * chartW;
        ctx.fillText(`t${data.snapshots[i].tick}`, x, h - 8);
    }

    // Lines
    resources.forEach((resource, ri) => {
        const color = CHART_COLORS[ri % CHART_COLORS.length];

        // Area fill
        ctx.beginPath();
        data.snapshots.forEach((snap, i) => {
            const x = padding.left + (i / (data.snapshots.length - 1 || 1)) * chartW;
            const y = padding.top + chartH - (snap.levels[resource] || 0) / maxVal * chartH;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        });
        const lastX = padding.left + chartW;
        ctx.lineTo(lastX, padding.top + chartH);
        ctx.lineTo(padding.left, padding.top + chartH);
        ctx.closePath();
        ctx.fillStyle = color + '10';
        ctx.fill();

        // Line
        ctx.beginPath();
        data.snapshots.forEach((snap, i) => {
            const x = padding.left + (i / (data.snapshots.length - 1 || 1)) * chartW;
            const y = padding.top + chartH - (snap.levels[resource] || 0) / maxVal * chartH;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        });
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.stroke();
    });

    // Legend
    const legendY = h - 4;
    let legendX = padding.left;
    ctx.font = '11px Inter';
    resources.forEach((r, i) => {
        const color = CHART_COLORS[i % CHART_COLORS.length];
        ctx.fillStyle = color;
        ctx.fillRect(legendX, legendY - 8, 12, 3);
        ctx.fillStyle = '#94a3b8';
        ctx.textAlign = 'left';
        ctx.fillText(r, legendX + 16, legendY - 3);
        legendX += ctx.measureText(r).width + 32;
    });
}

// === Entity Network ===
function drawEntityNetwork(world) {
    const canvas = document.getElementById('network-chart');
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;

    const rect = canvas.parentElement.getBoundingClientRect();
    const w = Math.min(rect.width - 40, 500);
    const h = 280;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, w, h);

    const entities = world.entities;
    const n = entities.length;
    const cx = w / 2;
    const cy = h / 2;
    const radius = Math.min(w, h) * 0.35;

    // Position entities in a circle
    const positions = entities.map((_, i) => ({
        x: cx + radius * Math.cos((2 * Math.PI * i) / n - Math.PI / 2),
        y: cy + radius * Math.sin((2 * Math.PI * i) / n - Math.PI / 2),
    }));

    // Draw links
    world.links.forEach(link => {
        const fromIdx = entities.indexOf(link.from);
        const toIdx = entities.indexOf(link.to);
        if (fromIdx === -1 || toIdx === -1) return;

        const from = positions[fromIdx];
        const to = positions[toIdx];

        ctx.beginPath();
        ctx.moveTo(from.x, from.y);
        ctx.lineTo(to.x, to.y);
        ctx.strokeStyle = 'rgba(99, 102, 241, 0.3)';
        ctx.lineWidth = 1.5;
        ctx.stroke();

        // Arrow
        const angle = Math.atan2(to.y - from.y, to.x - from.x);
        const arrowLen = 8;
        const mx = (from.x + to.x) / 2;
        const my = (from.y + to.y) / 2;
        ctx.beginPath();
        ctx.moveTo(mx, my);
        ctx.lineTo(mx - arrowLen * Math.cos(angle - 0.4), my - arrowLen * Math.sin(angle - 0.4));
        ctx.moveTo(mx, my);
        ctx.lineTo(mx - arrowLen * Math.cos(angle + 0.4), my - arrowLen * Math.sin(angle + 0.4));
        ctx.strokeStyle = 'rgba(139, 92, 246, 0.5)';
        ctx.lineWidth = 1.5;
        ctx.stroke();
    });

    // Draw nodes
    positions.forEach((pos, i) => {
        // Glow
        const gradient = ctx.createRadialGradient(pos.x, pos.y, 0, pos.x, pos.y, 20);
        gradient.addColorStop(0, 'rgba(99, 102, 241, 0.2)');
        gradient.addColorStop(1, 'rgba(99, 102, 241, 0)');
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, 20, 0, Math.PI * 2);
        ctx.fill();

        // Node
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, 8, 0, Math.PI * 2);
        ctx.fillStyle = CHART_COLORS[i % CHART_COLORS.length];
        ctx.fill();
        ctx.strokeStyle = 'rgba(255,255,255,0.2)';
        ctx.lineWidth = 1;
        ctx.stroke();

        // Label
        ctx.fillStyle = '#cbd5e1';
        ctx.font = '10px Inter';
        ctx.textAlign = 'center';
        const labelY = pos.y > cy ? pos.y + 22 : pos.y - 16;
        ctx.fillText(entities[i], pos.x, labelY);
    });
}

// === Event Distribution ===
function drawEventDistribution(data) {
    const canvas = document.getElementById('event-chart');
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;

    const rect = canvas.parentElement.getBoundingClientRect();
    const w = Math.min(rect.width - 40, 500);
    const h = 280;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, w, h);

    const types = Object.entries(data.eventTypeCounts);
    const maxCount = Math.max(...types.map(([, c]) => c));
    const barW = Math.min(60, (w - 80) / types.length - 16);
    const padding = { top: 20, bottom: 50, left: 55 };
    const chartH = h - padding.top - padding.bottom;

    const typeColors = {
        production: '#22c55e',
        transfer: '#3b82f6',
        shortage: '#ef4444',
        price: '#f59e0b',
    };

    types.forEach(([type, count], i) => {
        const barH = (count / maxCount) * chartH;
        const x = padding.left + i * (barW + 16) + 20;
        const y = padding.top + chartH - barH;
        const color = typeColors[type] || CHART_COLORS[i];

        // Bar gradient
        const grad = ctx.createLinearGradient(x, y, x, y + barH);
        grad.addColorStop(0, color);
        grad.addColorStop(1, color + '40');
        ctx.fillStyle = grad;

        // Rounded bar
        const r = 4;
        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.lineTo(x + barW - r, y);
        ctx.quadraticCurveTo(x + barW, y, x + barW, y + r);
        ctx.lineTo(x + barW, y + barH);
        ctx.lineTo(x, y + barH);
        ctx.lineTo(x, y + r);
        ctx.quadraticCurveTo(x, y, x + r, y);
        ctx.fill();

        // Count label
        ctx.fillStyle = '#cbd5e1';
        ctx.font = '11px "JetBrains Mono"';
        ctx.textAlign = 'center';
        ctx.fillText(count.toLocaleString(), x + barW / 2, y - 8);

        // Type label
        ctx.fillStyle = '#64748b';
        ctx.font = '11px Inter';
        ctx.fillText(type, x + barW / 2, h - 16);
    });
}

// === Objectives ===
function displayObjectives(objectives) {
    const container = document.getElementById('objectives-list');
    container.innerHTML = '';

    objectives.forEach(obj => {
        const card = document.createElement('div');
        const status = obj.status.toLowerCase();
        card.className = `objective-card ${status}`;
        card.innerHTML = `
            <div class="objective-icon">${status === 'passed' ? '✅' : status === 'failed' ? '❌' : '⏳'}</div>
            <div class="objective-name">${obj.name}: ${obj.status}</div>
        `;
        container.appendChild(card);
    });
}

// === Event Log ===
function displayEventLog(events) {
    const container = document.getElementById('event-log');
    container.innerHTML = '';

    const displayEvents = events.slice(0, 200);
    displayEvents.forEach(evt => {
        const entry = document.createElement('div');
        entry.className = 'log-entry';
        entry.innerHTML = `
            <span class="log-tick">t${evt.tick}</span>
            <span class="log-type ${evt.type}">${evt.type}</span>
            <span class="log-detail">${evt.entity} — ${evt.resource} ×${evt.amount}</span>
        `;
        container.appendChild(entry);
    });

    if (events.length > 200) {
        const more = document.createElement('div');
        more.className = 'log-empty';
        more.textContent = `... and ${events.length - 200} more events`;
        container.appendChild(more);
    }
}

function resetStats() {
    ['val-events', 'val-shortages', 'val-fingerprint', 'val-chain'].forEach(id => {
        document.getElementById(id).textContent = '—';
    });
    ['proof-world', 'proof-initial', 'proof-final', 'proof-chain'].forEach(id => {
        document.getElementById(id).textContent = '—';
    });
    document.getElementById('objectives-list').innerHTML = '<div class="objective-card pending"><div class="objective-icon">⏳</div><div class="objective-name">Awaiting simulation...</div></div>';
    document.getElementById('event-log').innerHTML = '<div class="log-empty">Run a simulation to see events</div>';
}

// === Event Filtering ===
document.getElementById('log-filter').addEventListener('input', (e) => {
    const filter = e.target.value.toLowerCase();
    document.querySelectorAll('.log-entry').forEach(entry => {
        entry.style.display = entry.textContent.toLowerCase().includes(filter) ? '' : 'none';
    });
});

document.getElementById('log-type-filter').addEventListener('change', (e) => {
    const type = e.target.value;
    document.querySelectorAll('.log-entry').forEach(entry => {
        const entryType = entry.querySelector('.log-type').textContent;
        entry.style.display = (type === 'all' || entryType === type) ? '' : 'none';
    });
});

// === Resource Select ===
document.getElementById('resource-select').addEventListener('change', () => {
    if (simulationData) drawResourceChart(simulationData);
});

// === Window Resize ===
let resizeTimeout;
window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
        const world = WORLDS[currentWorld];
        if (world) drawEntityNetwork(world);
        if (simulationData) {
            drawResourceChart(simulationData);
            drawEventDistribution(simulationData);
        }
    }, 150);
});

// === Initialize ===
switchWorld(currentWorld);
