// scripts.js

// dynamically get the year for the page footer
const currentYear = new Date().getFullYear();
const yearEl = document.getElementById("year");
if (yearEl) {
  yearEl.textContent = currentYear;
}