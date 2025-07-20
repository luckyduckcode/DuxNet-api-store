// DuxNet API Store Frontend
// This script provides the GUI interface for the DuxNet API Store

class DuxNetAPIStore {
    constructor() {
        this.baseUrl = 'http://localhost:8081';
        this.apiKey = 'demo-api-key-123';
        this.init();
    }

    async init() {
        await this.updateStatus();
        await this.loadServiceCategories();
        await this.loadServices();
        this.setupEventListeners();
        this.startStatusUpdates();
    }

    async updateStatus() {
        try {
            const response = await fetch(`${this.baseUrl}/api/status`);
            const status = await response.json();
            
            document.getElementById('nodeId').textContent = status.node_id.substring(0, 8) + '...';
            document.getElementById('did').textContent = status.did.substring(0, 20) + '...';
            document.getElementById('servicesCount').textContent = status.services_count;
            document.getElementById('reputationScore').textContent = status.reputation_score.toFixed(2);
            
            const statusIndicator = document.querySelector('.status-indicator');
            statusIndicator.style.background = status.is_online ? '#00ff00' : '#ff0000';
        } catch (error) {
            console.error('Failed to update status:', error);
        }
    }

    async loadServiceCategories() {
        try {
            const response = await fetch(`${this.baseUrl}/api/services/categories`);
            const data = await response.json();
            
            const categorySelect = document.getElementById('serviceCategory');
            categorySelect.innerHTML = '<option value="">Select Category</option>';
            
            data.categories.forEach(category => {
                const option = document.createElement('option');
                option.value = category;
                option.textContent = category;
                categorySelect.appendChild(option);
            });
        } catch (error) {
            console.error('Failed to load categories:', error);
        }
    }

    async loadServices() {
        try {
            const response = await fetch(`${this.baseUrl}/api/services/search`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.apiKey}`
                },
                body: JSON.stringify({
                    query: '',
                    limit: 10,
                    offset: 0,
                    sort_by: 'Name',
                    sort_order: 'asc'
                })
            });
            
            const data = await response.json();
            this.displayServices(data.services);
        } catch (error) {
            console.error('Failed to load services:', error);
        }
    }

    displayServices(services) {
        const servicesContainer = document.getElementById('servicesList');
        servicesContainer.innerHTML = '';
        
        if (services.length === 0) {
            servicesContainer.innerHTML = '<p class="no-services">No services found. Register your first service!</p>';
            return;
        }
        
        services.forEach(service => {
            const serviceCard = document.createElement('div');
            serviceCard.className = 'service-card';
            serviceCard.innerHTML = `
                <h3>${service.name}</h3>
                <p class="service-description">${service.description}</p>
                <div class="service-meta">
                    <span class="service-price">${this.formatPrice(service.price)}</span>
                    <span class="service-category">${service.categories.join(', ')}</span>
                </div>
                <div class="service-stats">
                    <span class="service-uptime">Uptime: ${service.uptime_percentage}%</span>
                    <span class="service-response">Response: ${service.response_time_ms}ms</span>
                </div>
            `;
            servicesContainer.appendChild(serviceCard);
        });
    }

    async registerService() {
        const formData = {
            name: document.getElementById('serviceName').value,
            description: document.getElementById('serviceDescription').value,
            price: parseInt(document.getElementById('servicePrice').value) || 0,
            categories: [document.getElementById('serviceCategory').value].filter(Boolean),
            tags: document.getElementById('serviceTags').value.split(',').map(tag => tag.trim()).filter(Boolean),
            sla: {
                uptime_guarantee: parseFloat(document.getElementById('uptimeGuarantee').value) || 99.0,
                max_response_time_ms: parseInt(document.getElementById('maxResponseTime').value) || 5000,
                support_response_hours: parseInt(document.getElementById('supportHours').value) || 24,
                refund_policy: { "PartialRefund": { "percentage": 50.0 } },
                availability_zones: ["global"]
            },
            version: document.getElementById('serviceVersion').value || "1.0.0",
            rate_limit_per_minute: parseInt(document.getElementById('rateLimit').value) || 1000,
            supported_formats: ["JSON"],
            examples: []
        };

        try {
            const response = await fetch(`${this.baseUrl}/api/services/register`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.apiKey}`
                },
                body: JSON.stringify(formData)
            });
            
            const result = await response.json();
            
            if (result.success) {
                this.showNotification('Service registered successfully!', 'success');
                this.clearForm();
                await this.loadServices();
                await this.updateStatus();
            } else {
                this.showNotification('Failed to register service: ' + result.message, 'error');
            }
        } catch (error) {
            console.error('Registration error:', error);
            this.showNotification('Failed to register service', 'error');
        }
    }

    async searchServices() {
        const query = document.getElementById('searchQuery').value;
        const category = document.getElementById('searchCategory').value;
        
        try {
            const response = await fetch(`${this.baseUrl}/api/services/search`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.apiKey}`
                },
                body: JSON.stringify({
                    query: query,
                    categories: category ? [category] : [],
                    limit: 20,
                    offset: 0,
                    sort_by: 'Name',
                    sort_order: 'asc'
                })
            });
            
            const data = await response.json();
            this.displayServices(data.services);
            
            const resultsCount = document.getElementById('searchResults');
            resultsCount.textContent = `Found ${data.total_count} services`;
        } catch (error) {
            console.error('Search error:', error);
            this.showNotification('Failed to search services', 'error');
        }
    }

    async loadAnalytics() {
        try {
            const response = await fetch(`${this.baseUrl}/api/analytics/usage`, {
                headers: {
                    'Authorization': `Bearer ${this.apiKey}`
                }
            });
            
            const data = await response.json();
            
            document.getElementById('totalRequests').textContent = data.total_requests;
            document.getElementById('periodHours').textContent = data.period_hours;
            
            // Update analytics chart or display
            this.updateAnalyticsDisplay(data);
        } catch (error) {
            console.error('Failed to load analytics:', error);
        }
    }

    updateAnalyticsDisplay(data) {
        // Simple analytics display - can be enhanced with charts
        const analyticsContainer = document.getElementById('analyticsDisplay');
        if (analyticsContainer) {
            analyticsContainer.innerHTML = `
                <div class="analytics-item">
                    <h4>Total Requests</h4>
                    <p>${data.total_requests}</p>
                </div>
                <div class="analytics-item">
                    <h4>Period</h4>
                    <p>${data.period_hours} hours</p>
                </div>
            `;
        }
    }

    formatPrice(price) {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: 'USD',
            minimumFractionDigits: 0
        }).format(price / 1000000); // Assuming price is in micro-units
    }

    clearForm() {
        document.getElementById('serviceName').value = '';
        document.getElementById('serviceDescription').value = '';
        document.getElementById('servicePrice').value = '';
        document.getElementById('serviceCategory').value = '';
        document.getElementById('serviceTags').value = '';
        document.getElementById('uptimeGuarantee').value = '';
        document.getElementById('maxResponseTime').value = '';
        document.getElementById('supportHours').value = '';
        document.getElementById('serviceVersion').value = '';
        document.getElementById('rateLimit').value = '';
    }

    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.remove();
        }, 3000);
    }

    setupEventListeners() {
        // Service registration
        const registerBtn = document.getElementById('registerService');
        if (registerBtn) {
            registerBtn.addEventListener('click', () => this.registerService());
        }

        // Service search
        const searchBtn = document.getElementById('searchServices');
        if (searchBtn) {
            searchBtn.addEventListener('click', () => this.searchServices());
        }

        // Analytics
        const analyticsBtn = document.getElementById('loadAnalytics');
        if (analyticsBtn) {
            analyticsBtn.addEventListener('click', () => this.loadAnalytics());
        }

        // Search input
        const searchInput = document.getElementById('searchQuery');
        if (searchInput) {
            searchInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.searchServices();
                }
            });
        }
    }

    startStatusUpdates() {
        setInterval(() => {
            this.updateStatus();
        }, 30000); // Update every 30 seconds
    }
}

// Initialize the application when the page loads
document.addEventListener('DOMContentLoaded', () => {
    window.duxNetStore = new DuxNetAPIStore();
});

// Add CSS for notifications
const style = document.createElement('style');
style.textContent = `
    .notification {
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 20px;
        border-radius: 10px;
        color: white;
        font-weight: 600;
        z-index: 1000;
        animation: slideIn 0.3s ease;
    }
    
    .notification.success {
        background: linear-gradient(45deg, #00ff00, #00cc00);
    }
    
    .notification.error {
        background: linear-gradient(45deg, #ff0000, #cc0000);
    }
    
    .notification.info {
        background: linear-gradient(45deg, #00ffff, #00cccc);
    }
    
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    
    .service-card {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 15px;
        padding: 20px;
        margin-bottom: 15px;
        transition: all 0.3s ease;
    }
    
    .service-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 5px 15px rgba(0, 255, 255, 0.2);
    }
    
    .service-card h3 {
        color: #00ffff;
        margin-bottom: 10px;
    }
    
    .service-description {
        color: #cccccc;
        margin-bottom: 15px;
    }
    
    .service-meta {
        display: flex;
        justify-content: space-between;
        margin-bottom: 10px;
    }
    
    .service-price {
        color: #ffff00;
        font-weight: 600;
    }
    
    .service-category {
        color: #ff00ff;
        font-size: 0.9rem;
    }
    
    .service-stats {
        display: flex;
        gap: 15px;
        font-size: 0.8rem;
        color: #888888;
    }
    
    .no-services {
        text-align: center;
        color: #888888;
        font-style: italic;
    }
    
    .analytics-item {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 10px;
        padding: 15px;
        margin-bottom: 10px;
        text-align: center;
    }
    
    .analytics-item h4 {
        color: #00ffff;
        margin-bottom: 5px;
    }
    
    .analytics-item p {
        font-size: 1.5rem;
        font-weight: 600;
        color: #ffffff;
    }
`;
document.head.appendChild(style); 