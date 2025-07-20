// DuxNet API Store Frontend JavaScript
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
        await this.loadUserData();
        await this.loadDashboardData();
        this.startRealTimeUpdates();
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

        // Service registration form
        const serviceForm = document.getElementById('serviceRegistrationForm');
        if (serviceForm) {
            serviceForm.addEventListener('submit', (e) => {
                e.preventDefault();
                this.registerService();
            });
        }

        // Search functionality
        const searchInput = document.getElementById('serviceSearch');
        if (searchInput) {
            searchInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.searchServices();
                }
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

        // Modal close functionality
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal')) {
                e.target.style.display = 'none';
            }
        });
    }

    switchTab(tabName) {
        // Hide all tab contents
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });

        // Remove active class from all nav links
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });

        // Show selected tab content
        const selectedTab = document.getElementById(tabName);
        if (selectedTab) {
            selectedTab.classList.add('active');
        }

        // Add active class to selected nav link
        const selectedLink = document.querySelector(`[data-tab="${tabName}"]`);
        if (selectedLink) {
            selectedLink.classList.add('active');
        }

        this.currentTab = tabName;

        // Load tab-specific data
        switch (tabName) {
            case 'dashboard':
                this.loadDashboardData();
                break;
            case 'marketplace':
                this.loadMarketplaceData();
                break;
            case 'developer':
                this.loadDeveloperData();
                break;
            case 'analytics':
                this.loadAnalyticsData();
                break;
            case 'network':
                this.loadNetworkData();
                break;
        }
    }

    // --- Wallet Management with Local Storage ---
    function getUserDuxAddress() {
        return localStorage.getItem('duxcoin_address') || '';
    }
    function setUserDuxAddress(addr) {
        localStorage.setItem('duxcoin_address', addr);
    }

    // Prompt user for DuxCoin address if not set
    function ensureUserDuxAddress() {
        let addr = getUserDuxAddress();
        if (!addr) {
            addr = prompt('Enter your DuxCoin address:');
            if (addr) setUserDuxAddress(addr);
        }
        return addr;
    }

    // --- Replace static user data with live API calls ---
    async loadUserData() {
        try {
            const addr = ensureUserDuxAddress();
            // Fetch wallet info and balance
            const [infoRes, balRes] = await Promise.all([
                fetch('http://localhost:8081/api/wallet/info'),
                fetch('http://localhost:8081/api/wallet/balances')
            ]);
            const info = await infoRes.json();
            const bal = await balRes.json();
            this.userData = {
                did: info.address || addr,
                balance: bal.balance || 0
            };
            this.updateUserInterface();
        } catch (error) {
            console.error('Error loading user data:', error);
            this.showNotification('Error loading user data', 'error');
        }
    }

    updateUserInterface() {
        if (this.userData) {
            document.querySelector('.user-did').textContent = this.userData.did;
            document.querySelector('.user-balance').textContent = `${this.userData.balance.toFixed(2)} DUX`;
        }
    }

    async loadDashboardData() {
        try {
            // Load status data
            const statusData = await this.fetchStatusData();
            this.updateStatusCards(statusData);

            // Load recent activity
            const activityData = await this.fetchActivityData();
            this.updateActivityList(activityData);

        } catch (error) {
            console.error('Error loading dashboard data:', error);
            this.showNotification('Error loading dashboard data', 'error');
        }
    }

    async fetchStatusData() {
        // Simulate API call
        return {
            activeServices: 156,
            p2pPeers: 23,
            totalRevenue: 1250.50,
            apiRequests: 15420
        };
    }

    updateStatusCards(data) {
        document.getElementById('activeServices').textContent = data.activeServices;
        document.getElementById('p2pPeers').textContent = data.p2pPeers;
        document.getElementById('totalRevenue').textContent = `${data.totalRevenue} DUX`;
        document.getElementById('apiRequests').textContent = data.apiRequests.toLocaleString();
    }

    async fetchActivityData() {
        // Simulate API call
        return [
            {
                id: 1,
                type: 'service_registered',
                title: 'New AI Service Registered',
                description: 'GPT-4 API service was registered by user123',
                time: '2 minutes ago',
                icon: 'fas fa-plus'
            },
            {
                id: 2,
                type: 'api_request',
                title: 'High API Usage Detected',
                description: 'Image processing service exceeded 1000 requests',
                time: '5 minutes ago',
                icon: 'fas fa-chart-line'
            },
            {
                id: 3,
                type: 'peer_connected',
                title: 'New P2P Peer Connected',
                description: 'Peer did:duxnet:peer456 joined the network',
                time: '10 minutes ago',
                icon: 'fas fa-network-wired'
            }
        ];
    }

    updateActivityList(activities) {
        const activityList = document.getElementById('activityList');
        if (!activityList) return;

        activityList.innerHTML = activities.map(activity => `
            <div class="activity-item">
                <div class="activity-icon">
                    <i class="${activity.icon}"></i>
                </div>
                <div class="activity-content">
                    <h4>${activity.title}</h4>
                    <p>${activity.description}</p>
                </div>
                <div class="activity-time">${activity.time}</div>
            </div>
        `).join('');
    }

    async loadMarketplaceData() {
        try {
            const services = await this.fetchServices();
            this.renderServiceGrid(services);
        } catch (error) {
            console.error('Error loading marketplace data:', error);
            this.showNotification('Error loading marketplace data', 'error');
        }
    }

    // --- Fetch live services from backend ---
    async fetchServices() {
        const res = await fetch('http://localhost:8081/api/services/search', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ query: '' })
        });
        const data = await res.json();
        return data.services || [];
    }

    renderServiceGrid(services) {
        const serviceGrid = document.getElementById('serviceGrid');
        if (!serviceGrid) return;

        serviceGrid.innerHTML = services.map(service => `
            <div class="service-card">
                <div class="service-header">
                    <div class="service-title">
                        <h3>${service.name}</h3>
                        <div class="service-metrics">
                            <span>⭐ ${service.rating}</span>
                            <span>📊 ${service.requests.toLocaleString()} requests</span>
                        </div>
                    </div>
                    <div class="service-price">${service.price} DUX</div>
                </div>
                <div class="service-description">${service.description}</div>
                <div class="service-categories">
                    ${service.categories.map(category => `
                        <span class="category-tag">${category}</span>
                    `).join('')}
                </div>
                <div class="service-metrics">
                    <span>Uptime: ${service.uptime}%</span>
                    <span>Response: ${service.responseTime}ms</span>
                    <span>v${service.version}</span>
                </div>
                <div class="service-actions">
                    <button class="btn-primary" onclick="duxnetFrontend.subscribeToService('${service.id}')">
                        Subscribe
                    </button>
                    <button class="btn-secondary" onclick="duxnetFrontend.viewServiceDetails('${service.id}')">
                        Details
                    </button>
                </div>
            </div>
        `).join('');
    }

    async loadDeveloperData() {
        try {
            this.renderApiKeys();
            await this.loadUsageAnalytics();
            this.loadRateLimitInfo();
            this.loadBillingInfo();
        } catch (error) {
            console.error('Error loading developer data:', error);
            this.showNotification('Error loading developer data', 'error');
        }
    }

    renderApiKeys() {
        const apiKeysList = document.getElementById('apiKeysList');
        if (!apiKeysList || !this.userData) return;

        apiKeysList.innerHTML = this.userData.apiKeys.map(key => `
            <div class="api-key-item">
                <div class="api-key-info">
                    <div class="api-key-name">${key.name}</div>
                    <div class="api-key-details">
                        Created: ${key.created} | Last used: ${key.lastUsed}
                    </div>
                </div>
                <div class="api-key-actions">
                    <button class="btn-small btn-secondary" onclick="duxnetFrontend.copyApiKey('${key.key}')">
                        Copy
                    </button>
                    <button class="btn-small btn-danger" onclick="duxnetFrontend.revokeApiKey('${key.id}')">
                        Revoke
                    </button>
                </div>
            </div>
        `).join('');
    }

    async loadUsageAnalytics() {
        // Simulate usage data
        const usageData = {
            totalRequests: 15420,
            successRate: 98.5,
            avgResponseTime: 450,
            requestsByHour: [120, 150, 180, 200, 220, 250, 280, 300, 320, 350, 380, 400]
        };

        document.getElementById('totalRequests').textContent = usageData.totalRequests.toLocaleString();
        document.getElementById('successRate').textContent = `${usageData.successRate}%`;
        document.getElementById('avgResponseTime').textContent = `${usageData.avgResponseTime}ms`;

        // Create usage chart
        this.createUsageChart(usageData.requestsByHour);
    }

    createUsageChart(data) {
        const canvas = document.getElementById('usageChart');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        
        // Simple chart drawing
        const width = canvas.width;
        const height = canvas.height;
        const maxValue = Math.max(...data);
        
        ctx.clearRect(0, 0, width, height);
        
        // Draw grid
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
        ctx.lineWidth = 1;
        
        for (let i = 0; i <= 4; i++) {
            const y = (height / 4) * i;
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
            ctx.stroke();
        }
        
        // Draw data line
        ctx.strokeStyle = '#00d4ff';
        ctx.lineWidth = 3;
        ctx.beginPath();
        
        data.forEach((value, index) => {
            const x = (width / (data.length - 1)) * index;
            const y = height - (value / maxValue) * height;
            
            if (index === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });
        
        ctx.stroke();
    }

    loadRateLimitInfo() {
        // Simulate rate limit data
        const rateLimitData = {
            currentLimit: '1000 requests/hour',
            usedRequests: 450,
            maxRequests: 1000
        };

        document.getElementById('currentRateLimit').textContent = rateLimitData.currentLimit;
        document.getElementById('usedRequests').textContent = rateLimitData.usedRequests;
        
        const progressPercentage = (rateLimitData.usedRequests / rateLimitData.maxRequests) * 100;
        document.getElementById('rateLimitProgress').style.width = `${progressPercentage}%`;
    }

    loadBillingInfo() {
        if (!this.userData) return;

        document.getElementById('currentBalance').textContent = `${this.userData.balance.toFixed(2)} DUX`;
        document.getElementById('monthlyUsage').textContent = '125.50 DUX';
        document.getElementById('nextBilling').textContent = 'Dec 1, 2024';
    }

    async loadAnalyticsData() {
        try {
            await this.loadUsageOverview();
            await this.loadTopEndpoints();
            await this.loadServicePerformance();
            await this.loadRevenueAnalytics();
        } catch (error) {
            console.error('Error loading analytics data:', error);
            this.showNotification('Error loading analytics data', 'error');
        }
    }

    async loadUsageOverview() {
        const canvas = document.getElementById('usageOverviewChart');
        if (!canvas) return;

        // Simulate usage overview data
        const data = {
            labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
            requests: [1200, 1400, 1600, 1800, 2000, 2200, 2400],
            revenue: [60, 70, 80, 90, 100, 110, 120]
        };

        this.createUsageOverviewChart(canvas, data);
    }

    createUsageOverviewChart(canvas, data) {
        const ctx = canvas.getContext('2d');
        const width = canvas.width;
        const height = canvas.height;
        
        ctx.clearRect(0, 0, width, height);
        
        // Draw requests line
        const maxRequests = Math.max(...data.requests);
        ctx.strokeStyle = '#00d4ff';
        ctx.lineWidth = 3;
        ctx.beginPath();
        
        data.requests.forEach((value, index) => {
            const x = (width / (data.requests.length - 1)) * index;
            const y = height - (value / maxRequests) * height * 0.8;
            
            if (index === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });
        
        ctx.stroke();
    }

    async loadTopEndpoints() {
        const topEndpoints = document.getElementById('topEndpoints');
        if (!topEndpoints) return;

        // Simulate endpoint data
        const endpoints = [
            { name: '/api/gpt/generate', requests: 5420, avgTime: 1200 },
            { name: '/api/image/process', requests: 3890, avgTime: 800 },
            { name: '/api/finance/quote', requests: 2340, avgTime: 200 },
            { name: '/api/blockchain/balance', requests: 1890, avgTime: 1500 }
        ];

        topEndpoints.innerHTML = endpoints.map(endpoint => `
            <div class="endpoint-item">
                <div class="endpoint-name">${endpoint.name}</div>
                <div class="endpoint-stats">
                    <span>${endpoint.requests.toLocaleString()} requests</span>
                    <span>${endpoint.avgTime}ms avg</span>
                </div>
            </div>
        `).join('');
    }

    async loadServicePerformance() {
        const servicePerformance = document.getElementById('servicePerformance');
        if (!servicePerformance) return;

        // Simulate performance data
        const services = [
            { name: 'GPT-4 API', uptime: 99.9, avgResponse: 1200, successRate: 98.5 },
            { name: 'Image Processing', uptime: 99.5, avgResponse: 800, successRate: 99.2 },
            { name: 'Financial Data', uptime: 99.8, avgResponse: 200, successRate: 99.8 }
        ];

        servicePerformance.innerHTML = services.map(service => `
            <div class="service-performance-item">
                <div class="service-name">${service.name}</div>
                <div class="service-stats">
                    <span>Uptime: ${service.uptime}%</span>
                    <span>Response: ${service.avgResponse}ms</span>
                    <span>Success: ${service.successRate}%</span>
                </div>
            </div>
        `).join('');
    }

    async loadRevenueAnalytics() {
        const canvas = document.getElementById('revenueChart');
        if (!canvas) return;

        // Simulate revenue data
        const data = {
            labels: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun'],
            revenue: [500, 650, 800, 950, 1100, 1250]
        };

        this.createRevenueChart(canvas, data);
    }

    createRevenueChart(canvas, data) {
        const ctx = canvas.getContext('2d');
        const width = canvas.width;
        const height = canvas.height;
        
        ctx.clearRect(0, 0, width, height);
        
        // Draw revenue bars
        const maxRevenue = Math.max(...data.revenue);
        const barWidth = width / data.revenue.length * 0.8;
        const barSpacing = width / data.revenue.length * 0.2;
        
        data.revenue.forEach((value, index) => {
            const x = index * (barWidth + barSpacing) + barSpacing / 2;
            const barHeight = (value / maxRevenue) * height * 0.8;
            const y = height - barHeight;
            
            ctx.fillStyle = '#00d4ff';
            ctx.fillRect(x, y, barWidth, barHeight);
        });
    }

    async loadNetworkData() {
        try {
            await this.loadNetworkStatus();
            await this.loadPeerList();
            await this.loadNetworkActivity();
        } catch (error) {
            console.error('Error loading network data:', error);
            this.showNotification('Error loading network data', 'error');
        }
    }

    async loadNetworkStatus() {
        // Simulate network status data
        const statusData = {
            connectedPeers: 23,
            networkHealth: 'Good',
            uptime: '72h 15m'
        };

        document.getElementById('connectedPeers').textContent = statusData.connectedPeers;
        document.getElementById('networkHealth').textContent = statusData.networkHealth;
        document.getElementById('networkUptime').textContent = statusData.uptime;
    }

    async loadPeerList() {
        const peerList = document.getElementById('peerList');
        if (!peerList) return;

        // Simulate peer data
        const peers = [
            { id: 'did:duxnet:peer123', status: 'Connected', latency: '45ms' },
            { id: 'did:duxnet:peer456', status: 'Connected', latency: '67ms' },
            { id: 'did:duxnet:peer789', status: 'Connected', latency: '89ms' },
            { id: 'did:duxnet:peer012', status: 'Connected', latency: '34ms' }
        ];

        peerList.innerHTML = peers.map(peer => `
            <div class="peer-item">
                <div class="peer-info">
                    <div class="peer-id">${peer.id}</div>
                    <div class="peer-status">${peer.status}</div>
                </div>
                <div class="peer-latency">${peer.latency}</div>
            </div>
        `).join('');
    }

    async loadNetworkActivity() {
        const canvas = document.getElementById('networkActivityChart');
        if (!canvas) return;

        // Simulate network activity data
        const data = {
            labels: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00'],
            connections: [15, 12, 18, 25, 30, 23],
            messages: [120, 95, 150, 200, 250, 180]
        };

        this.createNetworkActivityChart(canvas, data);
    }

    createNetworkActivityChart(canvas, data) {
        const ctx = canvas.getContext('2d');
        const width = canvas.width;
        const height = canvas.height;
        
        ctx.clearRect(0, 0, width, height);
        
        // Draw connections line
        const maxConnections = Math.max(...data.connections);
        ctx.strokeStyle = '#00d4ff';
        ctx.lineWidth = 3;
        ctx.beginPath();
        
        data.connections.forEach((value, index) => {
            const x = (width / (data.connections.length - 1)) * index;
            const y = height - (value / maxConnections) * height * 0.8;
            
            if (index === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });
        
        ctx.stroke();
    }

    // Service Management Methods
    async registerService() {
        const formData = new FormData(document.getElementById('serviceRegistrationForm'));
        const serviceData = {
            name: document.getElementById('serviceName').value,
            description: document.getElementById('serviceDescription').value,
            price: parseFloat(document.getElementById('servicePrice').value),
            version: document.getElementById('serviceVersion').value,
            categories: Array.from(document.getElementById('serviceCategories').selectedOptions).map(opt => opt.value),
            tags: document.getElementById('serviceTags').value.split(',').map(tag => tag.trim()),
            uptime_guarantee: parseFloat(document.getElementById('uptimeGuarantee').value),
            max_response_time: parseInt(document.getElementById('maxResponseTime').value),
            rate_limit: parseInt(document.getElementById('rateLimit').value),
            documentation_url: document.getElementById('documentationUrl').value
        };

        try {
            // Simulate API call
            console.log('Registering service:', serviceData);
            this.showNotification('Service registered successfully!', 'success');
            this.closeModal('serviceRegistrationModal');
            this.loadMarketplaceData();
        } catch (error) {
            console.error('Error registering service:', error);
            this.showNotification('Error registering service', 'error');
        }
    }

    async searchServices() {
        const query = document.getElementById('serviceSearch').value;
        if (!query.trim()) {
            this.loadMarketplaceData();
            return;
        }

        try {
            // Simulate search API call
            console.log('Searching for:', query);
            const services = await this.fetchServices();
            const filteredServices = services.filter(service => 
                service.name.toLowerCase().includes(query.toLowerCase()) ||
                service.description.toLowerCase().includes(query.toLowerCase()) ||
                service.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase()))
            );
            this.renderServiceGrid(filteredServices);
        } catch (error) {
            console.error('Error searching services:', error);
            this.showNotification('Error searching services', 'error');
        }
    }

    applyFilters() {
        const category = document.getElementById('categoryFilter').value;
        const sortBy = document.getElementById('sortFilter').value;
        const maxPrice = parseInt(document.getElementById('priceRange').value);

        this.fetchServices().then(services => {
            let filteredServices = services;

            // Apply category filter
            if (category) {
                filteredServices = filteredServices.filter(service => 
                    service.categories.includes(category)
                );
            }

            // Apply price filter
            filteredServices = filteredServices.filter(service => 
                service.price <= maxPrice
            );

            // Apply sorting
            switch (sortBy) {
                case 'popularity':
                    filteredServices.sort((a, b) => b.requests - a.requests);
                    break;
                case 'rating':
                    filteredServices.sort((a, b) => b.rating - a.rating);
                    break;
                case 'price':
                    filteredServices.sort((a, b) => a.price - b.price);
                    break;
                case 'price-desc':
                    filteredServices.sort((a, b) => b.price - a.price);
                    break;
                case 'newest':
                    // Simulate newest sorting
                    break;
            }

            this.renderServiceGrid(filteredServices);
        });
    }

    // Developer Portal Methods
    generateApiKey() {
        const keyName = prompt('Enter a name for your API key:');
        if (!keyName) return;

        try {
            // Simulate API key generation
            const newKey = {
                id: `key_${Date.now()}`,
                name: keyName,
                key: `dux_sk_${Math.random().toString(36).substr(2, 9)}`,
                created: new Date().toISOString().split('T')[0],
                lastUsed: 'Never'
            };

            this.userData.apiKeys.push(newKey);
            this.renderApiKeys();
            this.showNotification('API key generated successfully!', 'success');
        } catch (error) {
            console.error('Error generating API key:', error);
            this.showNotification('Error generating API key', 'error');
        }
    }

    copyApiKey(key) {
        navigator.clipboard.writeText(key).then(() => {
            this.showNotification('API key copied to clipboard!', 'success');
        }).catch(() => {
            this.showNotification('Error copying API key', 'error');
        });
    }

    revokeApiKey(keyId) {
        if (!confirm('Are you sure you want to revoke this API key?')) return;

        try {
            this.userData.apiKeys = this.userData.apiKeys.filter(key => key.id !== keyId);
            this.renderApiKeys();
            this.showNotification('API key revoked successfully!', 'success');
        } catch (error) {
            console.error('Error revoking API key:', error);
            this.showNotification('Error revoking API key', 'error');
        }
    }

    // Service Actions
    async subscribeToService(serviceId) {
        try {
            // Simulate subscription
            console.log('Subscribing to service:', serviceId);
            this.showNotification('Successfully subscribed to service!', 'success');
        } catch (error) {
            console.error('Error subscribing to service:', error);
            this.showNotification('Error subscribing to service', 'error');
        }
    }

    viewServiceDetails(serviceId) {
        // Simulate viewing service details
        console.log('Viewing service details:', serviceId);
        this.showNotification('Service details opened', 'info');
    }

    // Utility Methods
    showModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.style.display = 'block';
        }
    }

    closeModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.style.display = 'none';
        }
    }

    showNotification(message, type = 'info') {
        const notifications = document.getElementById('notifications');
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <i class="fas fa-${type === 'success' ? 'check-circle' : type === 'error' ? 'exclamation-circle' : 'info-circle'}"></i>
            ${message}
        `;

        notifications.appendChild(notification);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            notification.remove();
        }, 5000);
    }

    startRealTimeUpdates() {
        // Simulate real-time updates
        setInterval(() => {
            if (this.currentTab === 'dashboard') {
                this.loadDashboardData();
            }
        }, 30000); // Update every 30 seconds
    }

    // Quick Action Methods
    showServiceRegistration() {
        this.showModal('serviceRegistrationModal');
    }

    showServiceSearch() {
        this.switchTab('marketplace');
        document.getElementById('serviceSearch').focus();
    }

    showAnalytics() {
        this.switchTab('analytics');
    }

    showDeveloperPortal() {
        this.switchTab('developer');
    }
}

// --- Fetch live tasks for the user ---
async function fetchUserTasks() {
    const addr = getUserDuxAddress();
    const res = await fetch('http://localhost:8081/api/tasks', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
    });
    const data = await res.json();
    // Filter tasks by user address if needed
    return (data.tasks || []).filter(task => task.buyer_address === addr);
}

// --- Render live task list ---
DuxNetFrontend.prototype.renderTaskList = function(tasks) {
    const list = document.getElementById('taskList');
    if (!list) return;
    list.innerHTML = tasks.map(task => `
        <div class="task-item">
            <div>Task ID: ${task.id}</div>
            <div>Service: ${task.service_id}</div>
            <div>Status: ${task.status || 'Pending'}</div>
            <button onclick="showCompleteTaskModal('${task.id}')">Mark Complete & Release Escrow</button>
        </div>
    `).join('');
};

// --- Update purchase modal to use local storage address by default ---
function showPurchaseModal(service) {
    const modal = document.getElementById('purchaseModal');
    document.getElementById('purchaseServiceName').textContent = service.name;
    document.getElementById('purchaseServicePrice').textContent = service.price + ' DUX';
    document.getElementById('purchaseServiceId').value = service.id;
    document.getElementById('buyerDuxAddress').value = getUserDuxAddress();
    modal.style.display = 'block';
}

// --- On purchase, save address to local storage ---
async function submitPurchase() {
    const serviceId = document.getElementById('purchaseServiceId').value;
    const buyerAddress = document.getElementById('buyerDuxAddress').value;
    const payload = document.getElementById('purchaseTaskPayload').value;
    if (!serviceId || !buyerAddress) {
        alert('Please enter your DuxCoin address.');
        return;
    }
    setUserDuxAddress(buyerAddress);
    document.getElementById('purchaseStatus').textContent = 'Processing payment and task submission...';
    try {
        const res = await fetch('http://localhost:8081/api/tasks/submit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                service_id: serviceId,
                buyer_address: buyerAddress,
                payload: payload || ''
            })
        });
        const data = await res.json();
        if (data.success) {
            document.getElementById('purchaseStatus').textContent = 'Purchase successful! Task ID: ' + data.task_id;
        } else {
            document.getElementById('purchaseStatus').textContent = 'Error: ' + data.message;
        }
    } catch (e) {
        document.getElementById('purchaseStatus').textContent = 'Error: ' + e;
    }
}

// Provider: Show complete task modal
function showCompleteTaskModal(taskId) {
    const modal = document.getElementById('completeTaskModal');
    document.getElementById('completeTaskId').value = taskId;
    document.getElementById('completeTaskStatus').textContent = '';
    modal.style.display = 'block';
}

function closeCompleteTaskModal() {
    document.getElementById('completeTaskModal').style.display = 'none';
}

async function submitCompleteTask() {
    const taskId = document.getElementById('completeTaskId').value;
    document.getElementById('completeTaskStatus').textContent = 'Releasing escrow...';
    try {
        const res = await fetch(`http://localhost:8081/api/tasks/${taskId}/complete`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({})
        });
        const data = await res.json();
        if (data.success) {
            document.getElementById('completeTaskStatus').textContent = 'Escrow released! TXID: ' + data.txid;
        } else {
            document.getElementById('completeTaskStatus').textContent = 'Error: ' + data.message;
        }
    } catch (e) {
        document.getElementById('completeTaskStatus').textContent = 'Error: ' + e;
    }
}

// Patch renderServiceGrid to add Buy button
const originalRenderServiceGrid = DuxNetFrontend.prototype.renderServiceGrid;
DuxNetFrontend.prototype.renderServiceGrid = function(services) {
    const grid = document.getElementById('serviceGrid');
    if (!grid) return;
    grid.innerHTML = services.map(service => `
        <div class="service-card">
            <h3>${service.name}</h3>
            <p>${service.description}</p>
            <div class="service-meta">
                <span class="service-price">${service.price} DUX</span>
                <button class="buy-btn" onclick='showPurchaseModal(${JSON.stringify(service)})'>Buy</button>
            </div>
        </div>
    `).join('');
};

// Initialize the frontend when the page loads
let duxnetFrontend;
document.addEventListener('DOMContentLoaded', () => {
    duxnetFrontend = new DuxNetFrontend();
});

// Global functions for onclick handlers
window.showServiceRegistration = () => duxnetFrontend.showServiceRegistration();
window.showServiceSearch = () => duxnetFrontend.showServiceSearch();
window.showAnalytics = () => duxnetFrontend.showAnalytics();
window.showDeveloperPortal = () => duxnetFrontend.showDeveloperPortal();
window.searchServices = () => duxnetFrontend.searchServices();
window.generateApiKey = () => duxnetFrontend.generateApiKey();
window.copyApiKey = (key) => duxnetFrontend.copyApiKey(key);
window.revokeApiKey = (keyId) => duxnetFrontend.revokeApiKey(keyId);
window.subscribeToService = (serviceId) => duxnetFrontend.subscribeToService(serviceId);
window.viewServiceDetails = (serviceId) => duxnetFrontend.viewServiceDetails(serviceId);
window.closeModal = (modalId) => duxnetFrontend.closeModal(modalId); 