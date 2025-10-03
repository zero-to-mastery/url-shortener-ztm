// scripts.js

// dynamically get the year for the page footer
const currentYear = new Date().getFullYear();
const yearEl = document.getElementById("year");
if (yearEl) {
  yearEl.textContent = currentYear;
}

// URL Shortener functionality
class UrlShortener {
  constructor() {
    // Wait for DOM to be ready
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => this.init());
    } else {
      this.init();
    }
  }

  init() {
    this.form = document.getElementById('urlForm');
    this.urlInput = document.getElementById('urlInput');
    this.submitBtn = document.getElementById('submitBtn');
    this.result = document.getElementById('result');
    this.error = document.getElementById('error');
    this.shortUrl = document.getElementById('shortUrl');
    this.originalUrl = document.getElementById('originalUrl');
    this.copyBtn = document.getElementById('copyBtn');
    this.retryBtn = document.getElementById('retryBtn');
    
    if (!this.form || !this.urlInput || !this.submitBtn) {
      console.error('Required form elements not found!');
      return;
    }

    this.btnText = this.submitBtn.querySelector('.btn-text');
    this.btnLoading = this.submitBtn.querySelector('.btn-loading');
    this.copyText = this.copyBtn?.querySelector('.copy-text');
    this.copySuccess = this.copyBtn?.querySelector('.copy-success');
    
    // Add event listeners
    this.form.addEventListener('submit', (e) => this.handleSubmit(e));
    
    if (this.copyBtn) {
      this.copyBtn.addEventListener('click', () => this.copyToClipboard());
    }
    
    if (this.retryBtn) {
      this.retryBtn.addEventListener('click', () => this.clearResults());
    }
    
    this.urlInput.addEventListener('input', () => this.clearResults());
  }

  async handleSubmit(e) {
    e.preventDefault();
    
    const url = this.urlInput.value.trim();
    if (!url) return;

    this.setLoading(true);
    this.clearResults();

    try {
      const response = await fetch('/api/shorten', {
        method: 'POST',
        headers: {
          'Content-Type': 'text/plain',
          'X-API-Key': 'e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5'
        },
        body: url
      });

      if (response.ok) {
        const shortUrl = await response.text();
        this.showResult(url, shortUrl.trim());
      } else {
        const errorText = await response.text();
        this.showError(`Failed to shorten URL: ${errorText}`);
      }
    } catch (err) {
      console.error('Network error:', err);
      this.showError(`Network error: ${err.message}`);
    } finally {
      this.setLoading(false);
    }
  }

  setLoading(loading) {
    this.submitBtn.disabled = loading;
    if (loading) {
      this.btnText.style.display = 'none';
      this.btnLoading.style.display = 'flex';
    } else {
      this.btnText.style.display = 'inline';
      this.btnLoading.style.display = 'none';
    }
  }

  showResult(originalUrl, shortUrl) {
    this.shortUrl.value = shortUrl;
    this.originalUrl.textContent = originalUrl;
    this.result.style.display = 'block';
    
    // Scroll to result
    this.result.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  showError(message) {
    const errorMessage = document.getElementById('errorMessage');
    errorMessage.textContent = message;
    this.error.style.display = 'block';
    
    // Scroll to error
    this.error.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  clearResults() {
    this.result.style.display = 'none';
    this.error.style.display = 'none';
  }

  async copyToClipboard() {
    try {
      await navigator.clipboard.writeText(this.shortUrl.value);
      this.showCopySuccess();
    } catch (err) {
      // Fallback for older browsers
      this.shortUrl.select();
      document.execCommand('copy');
      this.showCopySuccess();
    }
  }

  showCopySuccess() {
    this.copyText.style.display = 'none';
    this.copySuccess.style.display = 'inline';
    
    setTimeout(() => {
      this.copyText.style.display = 'inline';
      this.copySuccess.style.display = 'none';
    }, 2000);
  }
}

// Initialize the URL shortener
new UrlShortener();

// Utility function to validate URLs
function isValidUrl(string) {
  try {
    new URL(string);
    return true;
  } catch (_) {
    return false;
  }
}