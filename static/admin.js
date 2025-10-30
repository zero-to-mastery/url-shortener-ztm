document.addEventListener("DOMContentLoaded", () => {
    initSidebar();
    setActiveLink();
    initDashboardChart();
    initAnalyticsCharts();
    handleAuthForms();
    initUserSearch();
    initUrlActions();
    initUserActions();
});

/**
 * Handles the mobile sidebar toggle functionality.
 */
function initSidebar() {
    const hamburger = document.getElementById("hamburger-button");
    const sidebar = document.getElementById("admin-sidebar");
    const contentOverlay = document.getElementById("content-overlay");

    if (hamburger && sidebar) {
        hamburger.addEventListener("click", () => {
            sidebar.classList.toggle("open");
            if (contentOverlay) {
                contentOverlay.classList.toggle("active");
            }
        });
    }

    if (contentOverlay) {
        contentOverlay.addEventListener("click", () => {
            sidebar.classList.remove("open");
            contentOverlay.classList.remove("active");
        });
    }
}

/**
 * Sets the 'active' class on the current navigation link based on the URL.
 */
function setActiveLink() {
    const currentPath = window.location.pathname;
    const navLinks = document.querySelectorAll(".sidebar-nav a");

    navLinks.forEach(link => {
        if (link.getAttribute("href") === currentPath) {
            link.classList.add("active");
        } else {
            link.classList.remove("active");
        }
    });
}

/**
 * Renders the main clicks chart on the admin dashboard.
 */
function initDashboardChart() {
    const ctx = document.getElementById('dashboardClicksChart');
    if (!ctx) return; // Only run on the dashboard page

    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
            datasets: [{
                label: 'Clicks This Week',
                data: [120, 190, 300, 500, 200, 350, 450],
                backgroundColor: 'rgba(76, 161, 163, 0.6)',
                borderColor: 'rgba(76, 161, 163, 1)',
                borderWidth: 1,
                borderRadius: 4,
            }]
        },
        options: {
            scales: {
                y: {
                    beginAtZero: true
                }
            },
            plugins: {
                legend: {
                    display: false // Hide the legend for a cleaner look
                }
            }
        }
    });
}

/**
 * Initializes charts on the analytics page using Chart.js.
 * This function only runs if it finds the chart canvas elements.
 */
function initAnalyticsCharts() {
    const clicksChartCtx = document.getElementById('clicksChart');
    const referrersChartCtx = document.getElementById('referrersChart');

    // If we're not on the analytics page, do nothing.
    if (!clicksChartCtx || !referrersChartCtx) {
        return;
    }

    // Sample data for the charts
    const clicksData = {
        labels: ['January', 'February', 'March', 'April', 'May', 'June', 'July'],
        datasets: [{
            label: 'Total Clicks',
            data: [65, 59, 80, 81, 56, 55, 40],
            fill: false,
            borderColor: 'rgb(76, 161, 163)',
            tension: 0.1
        }]
    };

    const referrersData = {
        labels: ['Google', 'Twitter / X', 'Direct', 'Facebook', 'Other'],
        datasets: [{
            label: 'Top Referrers',
            data: [300, 150, 100, 50, 120],
            backgroundColor: [
                '#4ca1a3',
                '#5dbab_c',
                '#7bcec_f',
                '#98e2e_3',
                '#b6f6f_8',
            ],
            hoverOffset: 4
        }]
    };

    new Chart(clicksChartCtx, {
        type: 'line',
        data: clicksData,
    });

    new Chart(referrersChartCtx, {
        type: 'doughnut',
        data: referrersData,
    });
}

/**
 * Handles front-end simulation of login and registration forms.
 * IMPORTANT: This is for UI demonstration only and is NOT secure.
 * The actual authentication must be handled by the backend.
 */
function handleAuthForms() {
    const loginForm = document.getElementById('loginForm');
    const registerForm = document.getElementById('registerForm');
    const messageDiv = document.getElementById('form-message');

    if (loginForm) {
        loginForm.addEventListener('submit', (e) => {
            e.preventDefault();
            displayMessage('Login successful! Redirecting...', 'success', messageDiv);
            // In a real app, you would make an API call here.
            setTimeout(() => window.location.href = '/admin', 1500);
        });
    }

    if (registerForm) {
        registerForm.addEventListener('submit', (e) => {
            e.preventDefault();
            displayMessage('Registration successful! Please log in.', 'success', messageDiv);
            // In a real app, you would make an API call here.
            setTimeout(() => window.location.href = '/admin/login', 1500);
        });
    }
}

function displayMessage(message, type, element) {
    if (!element) return;
    element.textContent = message;
    element.className = `form-message ${type}`; // 'success' or 'error'
}

/**
 * Handles the live search/filter functionality for the user management table.
 */
function initUserSearch() {
    const searchInput = document.getElementById('user-search');
    const userTableBody = document.getElementById('user-table-body');

    if (!searchInput || !userTableBody) {
        return; // Only run on the user management page
    }

    const tableRows = userTableBody.querySelectorAll('tr');

    searchInput.addEventListener('input', (e) => {
        const searchTerm = e.target.value.toLowerCase();

        tableRows.forEach(row => {
            const username = row.querySelector('td[data-label="Username"]').textContent.toLowerCase();
            const email = row.querySelector('td[data-label="Email"]').textContent.toLowerCase();

            if (username.includes(searchTerm) || email.includes(searchTerm)) {
                row.style.display = ''; // Show the row
            } else {
                row.style.display = 'none'; // Hide the row
            }
        });
    });
}

/**
 * Initializes copy and delete functionality for the URL management table.
 */
function initUrlActions() {
    const urlTableBody = document.getElementById('url-table-body');

    if (!urlTableBody) {
        return; // Only run on the URL management page
    }

    // Add event listeners to the action buttons
    urlTableBody.addEventListener('click', (e) => {
        const button = e.target.closest('button');
        if (!button) return;

        const row = e.target.closest('tr');
        if (!row) return;

        // Get the short code from the first cell
        const shortCodeCell = row.querySelector('.short-code');
        const originalUrlCell = row.querySelector('.url-cell a');
        if (!shortCodeCell || !originalUrlCell) return;

        const shortCode = shortCodeCell.textContent;
        const originalUrl = originalUrlCell.href;

        if (button.classList.contains('btn-sm') && !button.classList.contains('btn-danger')) {
            // Copy button clicked
            handleCopyUrl(originalUrl, shortCode);
        } else if (button.classList.contains('btn-sm') && button.classList.contains('btn-danger')) {
            // Delete button clicked
            handleDeleteUrl(shortCode, row);
        }
    });
}

/**
 * Handles the copy URL functionality
 */
function handleCopyUrl(originalUrl, shortCode) {
    const shortUrl = window.location.origin + '/' + shortCode;
    const fallbackUrl = originalUrl; // Use original URL as fallback if short code isn't available

    navigator.clipboard.writeText(shortUrl)
        .then(() => {
            // Show a visual feedback
            const buttons = document.querySelectorAll('.action-buttons');
            buttons.forEach(buttonGroup => {
                const copyBtn = buttonGroup.querySelector('.btn:not(.btn-danger)');
                if (copyBtn) {
                    const originalText = copyBtn.textContent;
                    copyBtn.textContent = 'Copied!';
                    setTimeout(() => {
                        copyBtn.textContent = originalText;
                    }, 2000);
                }
            });
        })
        .catch(err => {
            console.error('Failed to copy: ', err);
            // Fallback to original URL if the short code copy fails
            navigator.clipboard.writeText(fallbackUrl).catch(err => {
                console.error('Fallback copy also failed: ', err);
            });
        });
}

/**
 * Handles the delete URL functionality
 */
function handleDeleteUrl(shortCode, row) {
    if (confirm(`Are you sure you want to delete the short URL: ${shortCode}?`)) {
        // In a real application, you would make an API call here to delete the URL
        // fetch(`/api/urls/${shortCode}`, { method: 'DELETE' })
        //   .then(response => response.json())
        //   .then(data => {
        //     if (data.success) {
        //       row.remove(); // Remove the row from the table
        //     } else {
        //       alert('Failed to delete URL');
        //     }
        //   })
        //   .catch(error => {
        //     console.error('Error deleting URL:', error);
        //     alert('Error deleting URL');
        //   });

        // For demo purposes, just remove the row
        row.style.opacity = '0';
        setTimeout(() => {
            row.remove();
            // Show a message or update UI if table becomes empty
            const table = document.querySelector('.data-table');
            if (table && table.querySelectorAll('tbody tr').length === 0) {
                const emptyState = document.querySelector('.table-empty-state');
                if (emptyState) {
                    emptyState.classList.add('visible');
                }
            }
        }, 300);
    }
}

/**
 * Initializes edit and delete functionality for the user management table.
 */
function initUserActions() {
    const userTableBody = document.getElementById('user-table-body');

    if (!userTableBody) {
        return; // Only run on the user management page
    }

    // Add event listeners to the action buttons
    userTableBody.addEventListener('click', (e) => {
        const button = e.target.closest('button');
        if (!button) return;

        const row = e.target.closest('tr');
        if (!row) return;

        const username = row.querySelector('td[data-label="Username"]').textContent;
        const email = row.querySelector('td[data-label="Email"]').textContent;

        if (button.classList.contains('btn-sm') && !button.classList.contains('btn-danger')) {
            // Edit button clicked
            handleEditUser(username, email, row);
        } else if (button.classList.contains('btn-sm') && button.classList.contains('btn-danger')) {
            // Delete button clicked
            handleDeleteUser(username, email, row);
        }
    });
}

/**
 * Handles the edit user functionality
 */
function handleEditUser(username, email, row) {
    // In a real application, you would open a modal or redirect to an edit page
    alert(`Edit user: ${username} (${email})\nIn a real application, this would open an edit form.`);
}

/**
 * Handles the delete user functionality
 */
function handleDeleteUser(username, email, row) {
    if (confirm(`Are you sure you want to delete user: ${username} (${email})?`)) {
        // In a real application, you would make an API call here to delete the user
        // fetch(`/api/users/${username}`, { method: 'DELETE' })
        //   .then(response => response.json())
        //   .then(data => {
        //     if (data.success) {
        //       row.remove(); // Remove the row from the table
        //     } else {
        //       alert('Failed to delete user');
        //     }
        //   })
        //   .catch(error => {
        //     console.error('Error deleting user:', error);
        //     alert('Error deleting user');
        //   });

        // For demo purposes, just remove the row
        row.style.opacity = '0';
        setTimeout(() => {
            row.remove();
            // Show a message or update UI if table becomes empty
            const table = document.querySelector('.data-table');
            if (table && table.querySelectorAll('tbody tr').length === 0) {
                const emptyState = document.querySelector('.table-empty-state');
                if (emptyState) {
                    emptyState.classList.add('visible');
                }
            }
        }, 300);
    }
}