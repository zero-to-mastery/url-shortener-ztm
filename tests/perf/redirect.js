import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { scenario } from 'k6/execution';
import { Counter, Rate } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.4/index.js';

// Report dir (env overrideable). Ensure trailing slash for safe path joins.
const REPORT_PREFIX = __ENV.REPORT_PREFIX || './reports/redirect'; 
const URL_FILE = __ENV.URL_FILE

// ===== Data load (once) =====
// Use SharedArray to ensure large file is read once
const ids = new SharedArray("ids", () =>
  open(URL_FILE)
    .trim()
    .split("\n")
);

// ===== Custom metrics =====
const status308 = new Counter('status_308');
const status404 = new Counter('status_404');
const statusOther = new Counter('status_other');
const rate404   = new Rate('rate_404'); // 404 ratio

// ===== Scenario & thresholds (60s) =====
export const options = {
  discardResponseBodies: true,
  scenarios: {
    loop_test: {
      executor: 'constant-vus',
      vus: 750,          // concurrency
      duration: '60s',   // fixed 60s
    },
  },
  thresholds: {
    rate_404: ['rate>=0.23', 'rate<=0.27'], // expect ~25%
    http_req_duration: ['p(99)<300'],
  },
};

// ===== Main loop (runs for 60s) =====
export default function () {
  // Round-robin selection to avoid out-of-bounds
  const idx = Number(scenario.iterationInTest % ids.length);
  const id = ids[idx];
  const url = `http://127.0.0.1:8000/${id}`;

  const res = http.get(url, { redirects: 0 });

  const is308 = res.status === 308;
  const is404 = res.status === 404;

  // Counters
  if (is308) status308.add(1);
  else if (is404) status404.add(1);
  else statusOther.add(1);

  // 404 ratio
  rate404.add(is404);

  // Assertions
  check(res, {
    'status is 308 or 404': (r) => r.status === 308 || r.status === 404,
  });

  // Optional light throttling to protect local host
  // sleep(0.001);
}

// ===== Summary (includes totals & avg RPS) =====
export function handleSummary(data) {
  const m = data.metrics;

  const c308   = m['status_308']?.values?.count || 0;
  const c404   = m['status_404']?.values?.count || 0;
  const cOther = m['status_other']?.values?.count || 0;
  const total  = c308 + c404 + cOther;

  const p404 = total ? (c404 / total) * 100 : 0;
  const p308 = total ? (c308 / total) * 100 : 0;

  // Use runtime duration (ms) to compute avg RPS
  const durMs    = data?.state?.testRunDurationMs || 60000;
  const durSec   = durMs / 1000;
  const httpReqs = m['http_reqs']?.values?.count || total; // completed HTTP requests
  const avgRps   = durSec > 0 ? (httpReqs / durSec) : 0;

  const summaryText =
    `Status breakdown\n` +
    `----------------\n` +
    `308: ${c308} (${p308.toFixed(2)}%)\n` +
    `404: ${c404} (${p404.toFixed(2)}%)\n` +
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
        count_308: c308,
        count_404: c404,
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
