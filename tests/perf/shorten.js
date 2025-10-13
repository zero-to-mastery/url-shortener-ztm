import http from "k6/http";
import { check } from "k6";
import { SharedArray } from "k6/data";
import { scenario } from "k6/execution";
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.4/index.js';

// ── Environment variables ────────────────────────────────────────
// Convention: only use REPORT_PREFIX for naming all output artifacts
const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:8000';
const API_KEY = __ENV.API_KEY;
const REPORT_PREFIX = __ENV.REPORT_PREFIX || './reports/shorten'; // fallback to avoid errors
const URL_FILE = __ENV.URL_FILE

// ===== Data load (once) =====
// Use SharedArray to ensure large file is read once
const urls = new SharedArray("urls", () =>
  open(URL_FILE)
    .trim()
    .split("\n")
);

export const options = {
  discardResponseBodies: true,
  scenarios: {
    once_per_url: {
      executor: 'shared-iterations',
      vus: 300,
      iterations: urls.length,
      maxDuration: '10m',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<500'],
  },
};

if (!API_KEY) {
  throw new Error('API_KEY is not set (use -e API_KEY=...)');
}

export default function () {
  const idx = scenario.iterationInTest;
  if (idx >= urls.length) return;

  const payload = urls[idx];
  const headers = {
    'Content-Type': 'text/plain',
    'x-api-key': API_KEY,
  };

  const res = http.post(`${BASE_URL}/api/shorten`, payload, { headers });
  check(res, { 'status is 200': r => r.status === 200 });
}

// ── Unified summary output ──────────────────────────────────────
export function handleSummary(data) {
  return {
    // Text & JSON summaries under REPORT_PREFIX
    [`${REPORT_PREFIX}_summary.txt`]: textSummary(data, { indent: ' ', enableColors: true }),
    [`${REPORT_PREFIX}_summary.json`]: JSON.stringify(data),
  };
}
