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
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => this.init());
    } else {
      this.init();
    }
  }

  init() {
    this.form = document.getElementById("urlForm");
    this.urlInput = document.getElementById("urlInput");
    this.submitBtn = document.getElementById("submitBtn");
    this.result = document.getElementById("result");
    this.error = document.getElementById("error");
    this.shortUrl = document.getElementById("shortUrl");
    this.originalUrl = document.getElementById("originalUrl");
    this.copyBtn = document.getElementById("copyBtn");
    this.retryBtn = document.getElementById("retryBtn");

    if (!this.form || !this.urlInput || !this.submitBtn) {
      console.error("Required form elements not found!");
      return;
    }

    this.btnText = this.submitBtn.querySelector(".btn-text");
    this.btnLoading = this.submitBtn.querySelector(".btn-loading");
    this.copyText = this.copyBtn?.querySelector(".copy-text");
    this.copySuccess = this.copyBtn?.querySelector(".copy-success");

    // Add event listeners
    this.form.addEventListener("submit", (e) => this.handleSubmit(e));

    if (this.copyBtn) {
      this.copyBtn.addEventListener("click", () => this.copyToClipboard());
    }

    if (this.retryBtn) {
      this.retryBtn.addEventListener("click", () => this.clearResults());
    }

    document
      .getElementById("closeResult")
      ?.addEventListener("click", () => this.clearResults());
    document
      .getElementById("closeError")
      ?.addEventListener("click", () => this.clearResults());

    this.urlInput.addEventListener("input", () => this.clearResults());
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape") this.clearResults();
    });
  }

  async handleSubmit(e) {
    e.preventDefault();

    const url = this.urlInput.value.trim();
    if (!url) return;

    this.setLoading(true);
    this.clearResults();

    try {
      const response = await fetch("/api/public/shorten", {
        method: "POST",
        headers: {
          "Content-Type": "text/plain",
        },
        body: url,
      });

      if (response.ok) {
        const jsonResponse = await response.json();
        if (jsonResponse.success && jsonResponse.data) {
          this.showResult(
            jsonResponse.data.original_url,
            jsonResponse.data.shortened_url,
          );
        } else {
          this.showError(
            `Failed to shorten URL: ${jsonResponse.message || "Unknown error"}`,
          );
        }
      } else {
        // Try to parse JSON error response
        try {
          const errorResponse = await response.json();
          const errorMessage =
            errorResponse.message || "Unknown error occurred";
          this.showError(`Failed to shorten URL: ${errorMessage}`);
        } catch {
          const errorText = await response.text();
          this.showError(`Failed to shorten URL: ${errorText}`);
        }
      }
    } catch (err) {
      console.error("Network error:", err);
      this.showError(`Network error: ${err.message}`);
    } finally {
      this.setLoading(false);
    }
  }

  setLoading(loading) {
    this.submitBtn.disabled = loading;
    if (loading) {
      this.btnText.classList.add("hidden");
      this.btnLoading.classList.remove("hidden");
      this.submitBtn.disabled = true;
    } else {
      this.btnText.classList.remove("hidden");
      this.btnLoading.classList.add("hidden");
      this.submitBtn.disabled = false;
    }
  }

  showResult(originalUrl, shortUrl) {
    this.shortUrl.value = shortUrl;
    this.originalUrl.textContent = originalUrl;
    this.result.classList.remove("hidden");

    // Scroll to result
    this.result.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }

  showError(message) {
    const errorMessage = document.getElementById("errorMessage");
    errorMessage.textContent = message;
    this.error.classList.remove("hidden");

    // Scroll to error
    this.error.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }

  clearResults() {
    this.result.classList.add("hidden");
    this.error.classList.add("hidden");
  }

  async copyToClipboard() {
    try {
      await navigator.clipboard.writeText(this.shortUrl.value);
      this.showCopySuccess();
    } catch (err) {
      // Fallback for older browsers
      this.shortUrl.select();
      document.execCommand("copy");
      this.showCopySuccess();
    }
  }

  showCopySuccess() {
    this.copyText.classList.add("hidden");
    this.copySuccess.classList.remove("hidden");

    setTimeout(() => {
      this.copyText.classList.remove("hidden");
      this.copySuccess.classList.add("hidden");
    }, 2000);
  }
}


// ============================================
// ENHANCED FEATURES CAROUSEL WITH SIDE NAVIGATION
// ============================================

class FeaturesCarousel {
  constructor() {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => this.init());
    } else {
      this.init();
    }
  }

  init() {
    this.currentIndex = 0;
    this.autoRotateInterval = null;
    this.autoRotateDelay = 5000; // 5 seconds

    this.featureCards = document.querySelectorAll('[data-feature]');
    this.dots = document.querySelectorAll('.carousel-dots .dot');
    this.leftSideCard = document.querySelector('.feature-card-side--left');
    this.rightSideCard = document.querySelector('.feature-card-side--right');

    if (this.featureCards.length === 0 || this.dots.length === 0) {
      return;
    }

    // Add click handlers to dots
    this.dots.forEach((dot, index) => {
      dot.addEventListener('click', () => {
        this.goToSlide(index);
        this.resetAutoRotate();
      });
    });

    // Add click handlers to side cards for navigation
    if (this.leftSideCard) {
      this.leftSideCard.addEventListener('click', () => {
        this.previousSlide();
        this.resetAutoRotate();
      });
      this.leftSideCard.style.cursor = 'pointer';
      this.leftSideCard.setAttribute('role', 'button');
      this.leftSideCard.setAttribute('aria-label', 'Show previous feature');
      this.leftSideCard.setAttribute('tabindex', '0');
    }

    if (this.rightSideCard) {
      this.rightSideCard.addEventListener('click', () => {
        this.nextSlide();
        this.resetAutoRotate();
      });
      this.rightSideCard.style.cursor = 'pointer';
      this.rightSideCard.setAttribute('role', 'button');
      this.rightSideCard.setAttribute('aria-label', 'Show next feature');
      this.rightSideCard.setAttribute('tabindex', '0');
    }

    // Keyboard navigation for side cards
    [this.leftSideCard, this.rightSideCard].forEach((card, idx) => {
      if (card) {
        card.addEventListener('keydown', (e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            if (idx === 0) {
              this.previousSlide();
            } else {
              this.nextSlide();
            }
            this.resetAutoRotate();
          }
        });
      }
    });

    // Global keyboard navigation
    document.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowLeft') {
        this.previousSlide();
        this.resetAutoRotate();
      } else if (e.key === 'ArrowRight') {
        this.nextSlide();
        this.resetAutoRotate();
      }
    });

    this.addSwipeSupport();
    this.updateSideCards(); // Initial side cards update
    this.startAutoRotate();

    // Pause on hover
    const carouselWrapper = document.querySelector('.features-carousel');
    if (carouselWrapper) {
      carouselWrapper.addEventListener('mouseenter', () => this.stopAutoRotate());
      carouselWrapper.addEventListener('mouseleave', () => this.startAutoRotate());
    }
  }

   goToSlide(index, direction = null) {
    const prevIndex = this.currentIndex;
    
    // Determine direction if not specified
    if (direction === null) {
      if (index > prevIndex) {
        direction = 'next';
      } else if (index < prevIndex) {
        direction = 'prev';
      } else {
        return; // Same slide, no animation needed
      }
      
      // Handle wrap-around cases
      const totalCards = this.featureCards.length;
      if (prevIndex === totalCards - 1 && index === 0) {
        direction = 'next'; // Wrapping forward
      } else if (prevIndex === 0 && index === totalCards - 1) {
        direction = 'prev'; // Wrapping backward
      }
    }

    // Add exit animation to current card
    this.featureCards[prevIndex].classList.add(
      direction === 'next' ? 'slide-out-left' : 'slide-out-right'
    );

    // Update dots
    this.dots[prevIndex].classList.remove('active');
    this.dots[prevIndex].setAttribute('aria-selected', 'false');

    // Small delay to allow exit animation to start
    setTimeout(() => {
      // Hide previous card
      this.featureCards[prevIndex].classList.add('hidden');
      this.featureCards[prevIndex].classList.remove('slide-out-left', 'slide-out-right');

      // Show new card with entrance animation
      this.currentIndex = index;
      this.featureCards[this.currentIndex].classList.remove('hidden');
      this.featureCards[this.currentIndex].classList.add(
        direction === 'next' ? 'slide-in-right' : 'slide-in-left'
      );

      // Update dots
      this.dots[this.currentIndex].classList.add('active');
      this.dots[this.currentIndex].setAttribute('aria-selected', 'true');

      // Clean up animation classes
      setTimeout(() => {
        this.featureCards[this.currentIndex].classList.remove(
          'slide-in-left', 'slide-in-right'
        );
      }, 500);

      // Update side cards
      this.updateSideCards();
    }, 50);
  }

  updateSideCards() {
    const totalCards = this.featureCards.length;
    const prevIndex = (this.currentIndex - 1 + totalCards) % totalCards;
    const nextIndex = (this.currentIndex + 1) % totalCards;

    // Get preview content from prev/next cards
    const prevCard = this.featureCards[prevIndex];
    const nextCard = this.featureCards[nextIndex];

    // Update left side card with preview
    if (this.leftSideCard && prevCard) {
      const prevTitle = prevCard.querySelector('h3')?.textContent || '';
      const prevIcon = prevCard.querySelector('.feature-icon')?.textContent || '';
      
      this.leftSideCard.innerHTML = `
        <div class="side-card-content">
          <div class="side-card-icon">${prevIcon}</div>
          <h4 class="side-card-title">${prevTitle}</h4>
        </div>
      `;
    }

    // Update right side card with preview
    if (this.rightSideCard && nextCard) {
      const nextTitle = nextCard.querySelector('h3')?.textContent || '';
      const nextIcon = nextCard.querySelector('.feature-icon')?.textContent || '';
      
      this.rightSideCard.innerHTML = `
        <div class="side-card-content">
          <div class="side-card-icon">${nextIcon}</div>
          <h4 class="side-card-title">${nextTitle}</h4>
        </div>
      `;
    }
  }

  nextSlide() {
    const nextIndex = (this.currentIndex + 1) % this.featureCards.length;
    this.goToSlide(nextIndex);
  }

  previousSlide() {
    const prevIndex = (this.currentIndex - 1 + this.featureCards.length) % this.featureCards.length;
    this.goToSlide(prevIndex);
  }

  startAutoRotate() {
    this.stopAutoRotate();
    this.autoRotateInterval = setInterval(() => {
      this.nextSlide();
    }, this.autoRotateDelay);
  }

  stopAutoRotate() {
    if (this.autoRotateInterval) {
      clearInterval(this.autoRotateInterval);
      this.autoRotateInterval = null;
    }
  }

  resetAutoRotate() {
    this.stopAutoRotate();
    this.startAutoRotate();
  }

  addSwipeSupport() {
    let touchStartX = 0;
    let touchEndX = 0;
    const carouselWrapper = document.querySelector('.features-carousel');

    if (!carouselWrapper) return;

    carouselWrapper.addEventListener('touchstart', (e) => {
      touchStartX = e.changedTouches[0].screenX;
    }, { passive: true });

    carouselWrapper.addEventListener('touchend', (e) => {
      touchEndX = e.changedTouches[0].screenX;
      const swipeThreshold = 50;
      const diff = touchStartX - touchEndX;

      if (Math.abs(diff) > swipeThreshold) {
        if (diff > 0) {
          this.nextSlide();
        } else {
          this.previousSlide();
        }
        this.resetAutoRotate();
      }
    }, { passive: true });
  }
}

// Initialize carousel
new FeaturesCarousel();


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

// Admin sidebar toggle
const hamburger = document.getElementById("hamburger");
const sidebar = document.getElementById("adminNav");
const content = document.getElementById("adminContent");

// Toggle sidebar open/close
hamburger.addEventListener("click", (e) => {
  e.stopPropagation(); // Prevent event from bubbling up
  sidebar.classList.toggle("open");
  hamburger.classList.toggle("active");
  hamburger.textContent = sidebar.classList.contains("open") ? "✕" : "☰";
});

// Close sidebar when clicking outside
document.addEventListener("click", (e) => {
  if (
    sidebar.classList.contains("open") &&
    !sidebar.contains(e.target) &&
    !hamburger.contains(e.target)
  ) {
    sidebar.classList.remove("open");
    hamburger.classList.remove("active");
    hamburger.textContent = "☰";
  }
});
