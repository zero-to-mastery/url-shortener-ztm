// ./tests/perf/ops-runner.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { scenario } from 'k6/execution';
import { Counter, Rate } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.4/index.js';

// ── Environment variables ────────────────────────────────────────
// Convention: only use REPORT_PREFIX for naming all output artifacts
const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:8000';
const API_KEY = __ENV.API_KEY;
const REPORT_PREFIX = __ENV.REPORT_PREFIX || './reports/shorten'; // fallback to avoid errors
const NO_REPORT = __ENV.NO_REPORT
const FILE = __ENV.OP_FILE || './ops.txt';

const ops = new SharedArray('ops', () =>
    open(FILE)
        .trim()
        .split('\n')
        .map(line => line.trim())
        .filter(Boolean)
);

// ===== Custom metrics =====
const status200 = new Counter('status_200');
const status308 = new Counter('status_308');
const status404 = new Counter('status_404');
const status409 = new Counter('status_409');
const statusOther = new Counter('status_other');



export const options = {
    discardResponseBodies: true,
    scenarios: {
        once_per_url: {
            executor: 'shared-iterations',
            vus: 500,
            iterations: ops.length,
            maxDuration: '10m',
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<500'],
    },
    systemTags: ['status', 'method', 'name', 'scenario', 'check', 'expected_response'],
};

export default function () {
    const idx = scenario.iterationInTest;
    if (idx >= ops.length) return;

    const parts = ops[idx].split(' ');
    const method = parts[0];
    const path = parts[1];
    const body = parts[2] || null;
    const url = `${BASE_URL}${path}`;

    let name;
    if (method === 'GET') {
        name = 'GET /:code';
    } else if (path.startsWith('/api/shorten?alias=')) {
        name = 'POST /api/shorten?alias=:alias';
    } else if (path.startsWith('/api/shorten')) {
        name = 'POST /api/shorten';
    } else {
        name = `${method} ${path}`;
    }

    const params = {
        headers: {
            'Content-Type': 'text/plain',
            'x-api-key': API_KEY,
        },
        redirects: 0,
        tags: { name },
    };

    let res;
    if (method === 'GET') {
        res = http.get(url, params);
    } else if (method === 'POST') {
        res = http.post(url, body, params);
    }

    switch (res.status) {
        case 200: status200.add(1); break;
        case 308: status308.add(1); break;
        case 404: status404.add(1); break;
        case 409: status409.add(1); break;
        default: statusOther.add(1);
    }
}

export function handleSummary(data) {
    if (!NO_REPORT) {
        const m = data.metrics;

        const c200 = m['status_200']?.values?.count || 0;
        const c308 = m['status_308']?.values?.count || 0;
        const c404 = m['status_404']?.values?.count || 0;
        const c409 = m['status_409']?.values?.count || 0;
        const cOther = m['status_other']?.values?.count || 0;
        const total = c200 + c308 + c404 + c409 + cOther;

        const p200 = total ? (c200 / total) * 100 : 0;
        const p308 = total ? (c308 / total) * 100 : 0;
        const p404 = total ? (c404 / total) * 100 : 0;
        const p409 = total ? (c409 / total) * 100 : 0;

        // Use runtime duration (ms) to compute avg RPS
        const durMs = data?.state?.testRunDurationMs || 60000;
        const durSec = durMs / 1000;
        const httpReqs = m['http_reqs']?.values?.count || total; // completed HTTP requests
        const avgRps = durSec > 0 ? (httpReqs / durSec) : 0;

        const summaryText =
            `Status breakdown\n` +
            `----------------\n` +
            `200: ${c200} (${p200.toFixed(2)}%)\n` +
            `308: ${c308} (${p308.toFixed(2)}%)\n` +
            `404: ${c404} (${p404.toFixed(2)}%)\n` +
            `409: ${c409} (${p409.toFixed(2)}%)\n` +
            `Other: ${cOther}\n` +
            `Total: ${total}\n\n` +
            `Throughput\n` +
            `----------\n` +
            `http_reqs (completed): ${httpReqs}\n` +
            `avg_rps: ${avgRps.toFixed(2)} req/s (duration: ${durSec.toFixed(2)}s)\n`;


        return {
            // human-readable k6 summary
            [`${REPORT_PREFIX}_summary.txt`]: textSummary(data, { indent: ' ', enableColors: true }),
            // raw k6 JSON (full)
            [`${REPORT_PREFIX}_summary.json`]: JSON.stringify(data),
            // concise status breakdown
            [`${REPORT_PREFIX}_status_breakdown.txt`]: summaryText,
            [`${REPORT_PREFIX}_status_breakdown.json`]: JSON.stringify(
                {
                    count_200: c200,
                    count_308: c308,
                    count_404: c404,
                    count_409: c409,
                    count_other: cOther,
                    total,
                    http_reqs: httpReqs,
                    avg_rps: Number(avgRps.toFixed(4)),
                    duration_seconds: Number(durSec.toFixed(3)),
                },
                null,
                2
            ),
        };
    }
}