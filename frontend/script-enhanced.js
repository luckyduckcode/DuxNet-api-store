// DuxNet API Store Frontend JavaScript - Enhanced Version
class DuxNetFrontend {
    constructor() {
        this.apiBaseUrl = 'http://localhost:3000';
        this.currentTab = 'dashboard';
        this.apiKey = null;
        this.userData = null;
        this.charts = {};
        
        this.init();
    }

    async init() {
        this.setupEventListeners();
        this.setupScrollEffects();
        await this.loadUserData();
        await this.loadDashboardData();
        this.startRealTimeUpdates();
        this.animateCounters();
    }

    setupEventListeners() {
        // Tab navigation
        document.querySelectorAll('.nav-link').forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const tab = e.currentTarget.getAttribute('data-tab');
                this.switchTab(tab);
            });
        });

        // Search functionality
        const searchInput = document.getElementById('serviceSearch');
        if (searchInput) {
            searchInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.searchServices();
                }
            });
            
            // Add real-time search suggestions
            searchInput.addEventListener('input', (e) => {
                this.showSearchSuggestions(e.target.value);
            });
        }

        // Filters
        const categoryFilter = document.getElementById('categoryFilter');
        if (categoryFilter) {
            categoryFilter.addEventListener('change', () => this.applyFilters());
        }

        const sortFilter = document.getElementById('sortFilter');
        if (sortFilter) {
            sortFilter.addEventListener('change', () => this.applyFilters());
        }

        const priceRange = document.getElementById('priceRange');
        if (priceRange) {
            priceRange.addEventListener('input', (e) => {
                document.getElementById('priceValue').textContent = `Max: ${e.target.value} DUX`;
                this.applyFilters();
            });
        }

        // Add keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch(e.key) {
                    case 'k':
                        e.preventDefault();
                        document.getElementById('serviceSearch')?.focus();
                        break;
                    case '1':
                        e.preventDefault();
                        this.switchTab('dashboard');
                        break;
                    case '2':
                        e.preventDefault();
                        this.switchTab('marketplace');
                        break;
                    case '3':
                        e.preventDefault();
                        this.switchTab('developer');
                        break;
                }
            }
        });
    }

    setupScrollEffects() {
        const navbar = document.querySelector('.navbar');
        let lastScrollY = 0;

        window.addEventListener('scroll', () => {
            const currentScrollY = window.scrollY;
            
            // Add scrolled class for backdrop effect
            if (currentScrollY > 50) {
                navbar.classList.add('scrolled');
            } else {
                navbar.classList.remove('scrolled');
            }
            
            // Hide navbar on scroll down, show on scroll up
            if (currentScrollY > lastScrollY && currentScrollY > 100) {
                navbar.style.transform = 'translateY(-100%)';
            } else {
                navbar.style.transform = 'translateY(0)';
            }
            
            lastScrollY = currentScrollY;
        });

        // Parallax effect for hero section
        const heroVisual = document.querySelector('.hero-visual');
        if (heroVisual) {
            window.addEventListener('scroll', () => {
                const scrolled = window.pageYOffset;
                const parallax = scrolled * 0.5;
                heroVisual.style.transform = `translateY(${parallax}px)`;
            });
        }
    }

    animateCounters() {
        const counters = document.querySelectorAll('.stat-value, .hero-stat-value');
        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    this.animateCounter(entry.target);
                }
            });
        });

        counters.forEach(counter => observer.observe(counter));
    }

    animateCounter(element) {
        const target = parseInt(element.textContent.replace(/[^\d]/g, ''));
        const duration = 2000;
        const step = target / (duration / 16);
        let current = 0;

        const timer = setInterval(() => {
            current += step;
            if (current >= target) {
                current = target;
                clearInterval(timer);
            }
            
            const suffix = element.textContent.replace(/[\d,]/g, '');
            element.textContent = Math.floor(current).toLocaleString() + suffix;
        }, 16);
    }

    switchTab(tabName) {
        // Hide all tab contents with fade out
        document.querySelectorAll('.tab-content').forEach(content => {
            content.style.opacity = '0';
            setTimeout(() => {
                content.classList.remove('active');
            }, 150);
        });

        // Remove active class from all nav links
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });

        // Show selected tab content with fade in
        setTimeout(() => {
            const selectedTab = document.getElementById(tabName);
            if (selectedTab) {
                selectedTab.classList.add('active');
                selectedTab.style.opacity = '1';
            }

            // Add active class to selected nav link
            const selectedLink = document.querySelector(`[data-tab="${tabName}"]`);
            if (selectedLink) {
                selectedLink.classList.add('active');
            }
        }, 150);

        this.currentTab = tabName;

        // Load tab-specific data
        switch(tabName) {
            case 'dashboard':
                this.loadDashboardData();
                break;
            case 'marketplace':
                this.loadMarketplaceData();
                break;
            case 'analytics':
                this.loadAnalyticsData();
                break;
            case 'developer':
                this.loadDeveloperData();
                break;
            case 'network':
                this.loadNetworkData();
                break;
        }
    }

    // Show search suggestions
    showSearchSuggestions(query) {
        if (!query || query.length < 2) return;
        
        // Mock suggestions - in real app this would come from API
        const suggestions = [
            'AI Text Processing',
            'Data Analytics',
            'Blockchain Oracle',
            'Image Recognition',
            'Natural Language Processing'
        ].filter(s => s.toLowerCase().includes(query.toLowerCase()));
        
        // Create suggestions dropdown (implement as needed)
        console.log('Suggestions:', suggestions);
    }

    // Load marketplace data
    async loadMarketplaceData() {
        try {
            // This would normally fetch from your API
            const mockServices = [
                {
                    id: 'service-1',
                    name: 'AI Text Analyzer',
                    description: 'Advanced natural language processing with sentiment analysis',
                    category: 'ai',
                    rating: 4.9,
                    users: 1200,
                    uptime: 99.9,
                    price: '50 DUX/1k requests',
                    featured: true
                },
                {
                    id: 'service-2',
                    name: 'Data Transformer',
                    description: 'Real-time data processing and format conversion',
                    category: 'data',
                    rating: 4.7,
                    users: 856,
                    uptime: 98.5,
                    price: '25 DUX/1k requests',
                    trending: true
                },
                {
                    id: 'service-3',
                    name: 'Blockchain Oracle',
                    description: 'Secure and reliable off-chain data for smart contracts',
                    category: 'blockchain',
                    rating: 4.8,
                    users: 432,
                    uptime: 99.2,
                    price: '100 DUX/1k requests',
                    new: true
                }
            ];
            
            this.renderServices(mockServices);
        } catch (error) {
            console.error('Error loading marketplace data:', error);
        }
    }

    // Render services in the grid
    renderServices(services) {
        const grid = document.getElementById('servicesGrid');
        if (!grid) return;

        grid.innerHTML = services.map(service => `
            <div class="service-card" data-service-id="${service.id}">
                <div class="service-header">
                    <div class="service-icon ${service.category}">
                        <i class="fas fa-${this.getCategoryIcon(service.category)}"></i>
                    </div>
                    ${service.featured ? '<div class="service-badge featured">Featured</div>' : ''}
                    ${service.trending ? '<div class="service-badge trending">Trending</div>' : ''}
                    ${service.new ? '<div class="service-badge new">New</div>' : ''}
                </div>
                <div class="service-content">
                    <h3 class="service-title">${service.name}</h3>
                    <p class="service-description">${service.description}</p>
                    <div class="service-stats">
                        <span class="stat"><i class="fas fa-star"></i> ${service.rating}</span>
                        <span class="stat"><i class="fas fa-users"></i> ${service.users}</span>
                        <span class="stat"><i class="fas fa-bolt"></i> ${service.uptime}%</span>
                    </div>
                </div>
                <div class="service-footer">
                    <div class="service-price">${service.price}</div>
                    <button class="btn-primary service-connect" onclick="duxnet.connectToService('${service.id}')">
                        Connect
                    </button>
                </div>
            </div>
        `).join('');
    }

    // Get category icon
    getCategoryIcon(category) {
        const icons = {
            'ai': 'brain',
            'data': 'database',
            'blockchain': 'link',
            'utility': 'tools',
            'analytics': 'chart-line'
        };
        return icons[category] || 'cube';
    }

    // Connect to service
    async connectToService(serviceId) {
        console.log('Connecting to service:', serviceId);
        // Implementation for service connection
        this.showNotification('Connecting to service...', 'info');
    }

    // Search services
    async searchServices() {
        const query = document.getElementById('serviceSearch').value;
        console.log('Searching for:', query);
        // Implementation for search
    }

    // Apply filters
    applyFilters() {
        const category = document.getElementById('categoryFilter').value;
        const sort = document.getElementById('sortFilter').value;
        const maxPrice = document.getElementById('priceRange').value;
        
        console.log('Applying filters:', { category, sort, maxPrice });
        // Implementation for filtering
    }

    // Load analytics data
    async loadAnalyticsData() {
        console.log('Loading analytics data...');
        // Implementation for analytics
    }

    // Load developer data
    async loadDeveloperData() {
        console.log('Loading developer data...');
        // Implementation for developer portal
    }

    // Load network data
    async loadNetworkData() {
        console.log('Loading network data...');
        // Implementation for P2P network view
    }

    // Load user data
    async loadUserData() {
        try {
            // Mock user data
            this.userData = {
                did: 'did:duxnet:user123',
                balance: 500.00,
                avatar: null
            };
            this.updateUserInterface();
        } catch (error) {
            console.error('Error loading user data:', error);
        }
    }

    // Update user interface
    updateUserInterface() {
        const didElement = document.querySelector('.user-did');
        const balanceElement = document.querySelector('.user-balance');
        
        if (didElement && this.userData) {
            didElement.textContent = this.userData.did;
        }
        
        if (balanceElement && this.userData) {
            balanceElement.textContent = `${this.userData.balance.toFixed(2)} DUX`;
        }
    }

    // Load dashboard data
    async loadDashboardData() {
        try {
            await this.fetchStatusData();
            await this.fetchActivityData();
        } catch (error) {
            console.error('Error loading dashboard data:', error);
        }
    }

    // Fetch status data
    async fetchStatusData() {
        // Mock status data
        const data = {
            activeServices: 42,
            p2pPeers: 247,
            totalRevenue: 1247,
            apiRequests: 1200000
        };
        
        this.updateStatusCards(data);
    }

    // Update status cards
    updateStatusCards(data) {
        const elements = {
            activeServices: document.getElementById('activeServices'),
            p2pPeers: document.getElementById('p2pPeers'),
            totalRevenue: document.getElementById('totalRevenue'),
            apiRequests: document.getElementById('apiRequests')
        };

        if (elements.activeServices) elements.activeServices.textContent = data.activeServices;
        if (elements.p2pPeers) elements.p2pPeers.textContent = data.p2pPeers;
        if (elements.totalRevenue) elements.totalRevenue.textContent = `${data.totalRevenue} DUX`;
        if (elements.apiRequests) elements.apiRequests.textContent = this.formatNumber(data.apiRequests);
    }

    // Fetch activity data
    async fetchActivityData() {
        // Mock activity data - already in HTML
        console.log('Activity data loaded');
    }

    // Format number for display
    formatNumber(num) {
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1) + 'M';
        } else if (num >= 1000) {
            return (num / 1000).toFixed(1) + 'k';
        }
        return num.toString();
    }

    // Start real-time updates
    startRealTimeUpdates() {
        setInterval(() => {
            if (this.currentTab === 'dashboard') {
                this.fetchStatusData();
            }
        }, 30000); // Update every 30 seconds
    }

    // Show notification
    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message;
        
        // Style the notification
        Object.assign(notification.style, {
            position: 'fixed',
            top: '20px',
            right: '20px',
            background: type === 'success' ? 'var(--success-color)' : 
                       type === 'error' ? 'var(--danger-color)' : 'var(--primary-color)',
            color: 'white',
            padding: '15px 20px',
            borderRadius: 'var(--radius-lg)',
            boxShadow: 'var(--shadow-lg)',
            zIndex: '10000',
            opacity: '0',
            transform: 'translateX(100%)',
            transition: 'all 0.3s ease'
        });

        document.body.appendChild(notification);

        // Animate in
        setTimeout(() => {
            notification.style.opacity = '1';
            notification.style.transform = 'translateX(0)';
        }, 10);

        // Remove after 3 seconds
        setTimeout(() => {
            notification.style.opacity = '0';
            notification.style.transform = 'translateX(100%)';
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }
}

// Global functions for onclick handlers
window.showServiceRegistration = function() {
    console.log('Show service registration modal');
    // Implementation for service registration modal
};

window.loadMoreServices = function() {
    console.log('Load more services');
    // Implementation for loading more services
};

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.duxnet = new DuxNetFrontend();
});
