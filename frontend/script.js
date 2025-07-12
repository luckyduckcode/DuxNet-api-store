// DuxNet Frontend JavaScript - Tauri Integration
// Check if we're running in Tauri
const isTauri = typeof window !== 'undefined' && window.__TAURI__;

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    if (isTauri) {
        console.log('Running in Tauri desktop app');
    loadNodeStatus();
    refreshStats();
    refreshBalances();
    loadWalletAddresses();
    
    // Auto-refresh stats every 30 seconds
    setInterval(refreshStats, 30000);
    
    // Auto-refresh balances every 60 seconds
    setInterval(refreshBalances, 60000);
    } else {
        console.log('Running in web browser - using HTTP API');
        // Fallback to HTTP API for web browser
        loadNodeStatus();
        refreshStats();
        refreshBalances();
        loadWalletAddresses();
        
        setInterval(refreshStats, 30000);
        setInterval(refreshBalances, 60000);
    }
});

// Load node status
async function loadNodeStatus() {
    try {
        if (isTauri) {
            const status = await window.__TAURI__.invoke('get_network_status');
            if (status.success) {
                document.getElementById('nodeDid').textContent = 'DuxNet Node';
                document.getElementById('nodeReputation').textContent = '5.00';
                document.getElementById('peerCount').textContent = status.peers || 0;
            }
        } else {
            const response = await fetch('http://localhost:8081/api/status');
        const status = await response.json();
        
        document.getElementById('nodeDid').textContent = status.did;
        document.getElementById('nodeReputation').textContent = status.reputation_score.toFixed(2);
        document.getElementById('peerCount').textContent = status.peers_count;
        }
    } catch (error) {
        console.error('Failed to load node status:', error);
        showNotification('Failed to load node status', 'error');
    }
}

// Register a new service
async function registerService() {
    const name = document.getElementById('serviceName').value;
    const description = document.getElementById('serviceDescription').value;
    const price = parseFloat(document.getElementById('servicePrice').value);
    const currency = document.getElementById('serviceCurrency').value;
    
    if (!name || !description || !price) {
        showNotification('Please fill in all fields', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            // For now, show a notification that service registration is not yet implemented in Tauri
            showNotification('Service registration will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/services/register', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                name,
                description,
                price,
                currency
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
            showNotification(`Service registered successfully! ID: ${result.service_id}`, 'success');
            
            // Clear form
            document.getElementById('serviceName').value = '';
            document.getElementById('serviceDescription').value = '';
            document.getElementById('servicePrice').value = '';
            document.getElementById('serviceCurrency').value = 'USDC';
        } else {
            showNotification(result.message, 'error');
            }
        }
        
    } catch (error) {
        console.error('Failed to register service:', error);
        showNotification('Failed to register service', 'error');
    }
}

// Search for services
async function searchServices() {
    const query = document.getElementById('searchQuery').value;
    
    if (!query) {
        showNotification('Please enter a search query', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            const result = await window.__TAURI__.invoke('get_services');
            if (result.success) {
                // Filter services based on query
                const filteredServices = result.services.filter(service => 
                    service.name.toLowerCase().includes(query.toLowerCase()) ||
                    service.description.toLowerCase().includes(query.toLowerCase())
                );
                displaySearchResults(filteredServices);
                showNotification(`Found ${filteredServices.length} services`, 'success');
            }
        } else {
            const response = await fetch('http://localhost:8081/api/services/search', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
            displaySearchResults(result.services);
            showNotification(`Found ${result.services.length} services`, 'success');
        } else {
            showNotification(result.message, 'error');
            }
        }
        
    } catch (error) {
        console.error('Failed to search services:', error);
        showNotification('Failed to search services', 'error');
    }
}

// Display search results
function displaySearchResults(services) {
    const container = document.getElementById('searchResults');
    
    if (services.length === 0) {
        container.innerHTML = '<div class="result-item"><p>No services found</p></div>';
        return;
    }
    
    container.innerHTML = services.map(service => `
        <div class="result-item">
            <h4>${service.name}</h4>
            <p><strong>ID:</strong> ${service.id}</p>
            <p><strong>Description:</strong> ${service.description}</p>
            <p><strong>Price:</strong> ${service.price} ${service.currency || 'DOGE'}</p>
            <p><strong>Provider:</strong> ${service.provider_did}</p>
            <p><strong>Reputation:</strong> ${service.reputation_score.toFixed(2)}</p>
        </div>
    `).join('');
}

// Submit a task
async function submitTask() {
    const serviceId = document.getElementById('taskService').value;
    const payload = document.getElementById('taskPayload').value;
    const cpuCores = parseInt(document.getElementById('taskCpu').value);
    const memoryMb = parseInt(document.getElementById('taskMemory').value);
    const timeoutSeconds = parseInt(document.getElementById('taskTimeout').value);
    
    if (!serviceId || !payload) {
        showNotification('Please fill in service ID and payload', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            showNotification('Task submission will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/tasks/submit', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                service_id: serviceId,
                payload,
                cpu_cores: cpuCores,
                memory_mb: memoryMb,
                timeout_seconds: timeoutSeconds
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
            showNotification(`Task submitted successfully! ID: ${result.task_id}`, 'success');
            
            // Clear form
            document.getElementById('taskService').value = '';
            document.getElementById('taskPayload').value = '';
            document.getElementById('taskCpu').value = '1';
            document.getElementById('taskMemory').value = '512';
            document.getElementById('taskTimeout').value = '60';
        } else {
            showNotification(result.message, 'error');
            }
        }
        
    } catch (error) {
        console.error('Failed to submit task:', error);
        showNotification('Failed to submit task', 'error');
    }
}

// Create an escrow
async function createEscrow() {
    const serviceId = document.getElementById('escrowService').value;
    const sellerDid = document.getElementById('escrowSeller').value;
    const amount = parseFloat(document.getElementById('escrowAmount').value);
    const currency = document.getElementById('escrowCurrency').value;
    
    if (!serviceId || !sellerDid || !amount) {
        showNotification('Please fill in all fields', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            showNotification('Escrow creation will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/escrow/create', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                service_id: serviceId,
                seller_did: sellerDid,
                amount,
                currency
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
            showNotification(`Escrow created successfully! ID: ${result.escrow_id}`, 'success');
            
            // Clear form
            document.getElementById('escrowService').value = '';
            document.getElementById('escrowSeller').value = '';
            document.getElementById('escrowAmount').value = '';
                document.getElementById('escrowCurrency').value = 'USDC';
        } else {
            showNotification(result.message, 'error');
            }
        }
        
    } catch (error) {
        console.error('Failed to create escrow:', error);
        showNotification('Failed to create escrow', 'error');
    }
}

// Refresh network statistics
async function refreshStats() {
    try {
        if (isTauri) {
            const status = await window.__TAURI__.invoke('get_network_status');
            if (status.success) {
                document.getElementById('totalPeers').textContent = status.peers || 0;
                document.getElementById('totalServices').textContent = status.services || 0;
                document.getElementById('totalTasks').textContent = status.tasks || 0;
            }
        } else {
            const response = await fetch('http://localhost:8081/api/stats');
        const stats = await response.json();
        
            document.getElementById('totalPeers').textContent = stats.total_peers;
            document.getElementById('totalServices').textContent = stats.total_services;
            document.getElementById('totalTasks').textContent = stats.total_tasks;
        }
    } catch (error) {
        console.error('Failed to refresh stats:', error);
    }
}

// Show notification
function showNotification(message, type) {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    
    // Style the notification
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 20px;
        border-radius: 10px;
        color: white;
        font-weight: 600;
        z-index: 1000;
        animation: slideIn 0.3s ease;
        max-width: 300px;
    `;
    
    // Set background color based on type
    switch (type) {
        case 'success':
            notification.style.background = 'linear-gradient(45deg, #00ff00, #00cc00)';
            break;
        case 'error':
            notification.style.background = 'linear-gradient(45deg, #ff0000, #cc0000)';
            break;
        case 'info':
            notification.style.background = 'linear-gradient(45deg, #0066ff, #0044cc)';
            break;
        default:
            notification.style.background = 'linear-gradient(45deg, #ffff00, #cccc00)';
            notification.style.color = '#000';
    }
    
    document.body.appendChild(notification);
    
    // Remove notification after 5 seconds
    setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 300);
    }, 5000);
}

// Add CSS animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
`;
document.head.appendChild(style);

// Simulate API call for testing
async function simulateApiCall() {
    showNotification('Simulating API call...', 'info');
    await new Promise(resolve => setTimeout(resolve, 1000));
    showNotification('API call completed successfully!', 'success');
}

// Refresh wallet balances
async function refreshBalances() {
    try {
        if (isTauri) {
            const result = await window.__TAURI__.invoke('get_balances');
            if (result.success) {
                displayBalances(result.balances);
            }
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/balances');
        const balances = await response.json();
            displayBalances(balances);
        }
    } catch (error) {
        console.error('Failed to refresh balances:', error);
    }
}

// Display balances
function displayBalances(balances) {
    const container = document.getElementById('balancesContainer');
    if (!container) return;
    
    container.innerHTML = Object.entries(balances).map(([currency, amount]) => `
        <div class="balance-item">
            <span class="currency">${currency}</span>
            <span class="amount">${formatCurrencyAmount(amount, currency)}</span>
        </div>
    `).join('');
}

// Change preferred currency
async function changePreferredCurrency() {
    const currency = document.getElementById('preferredCurrency').value;
    
    try {
        if (isTauri) {
            showNotification(`Preferred currency changed to ${currency}`, 'success');
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/preferred-currency', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                currency
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
            showNotification(`Preferred currency changed to ${currency}`, 'success');
        } else {
            showNotification(result.message, 'error');
            }
        }
    } catch (error) {
        console.error('Failed to change preferred currency:', error);
        showNotification('Failed to change preferred currency', 'error');
    }
}

// Format currency amount
function formatCurrencyAmount(amount, currency) {
    const decimals = getCurrencyDecimals(currency);
    return parseFloat(amount).toFixed(decimals);
}

// Get currency decimals
function getCurrencyDecimals(currency) {
    const decimals = {
        'BTC': 8,
        'ETH': 18,
        'USDC': 6,
        'LTC': 8,
        'XMR': 12,
        'DOGE': 8
    };
    return decimals[currency] || 2;
}

// Show wallet tab
function showWalletTab(tabName) {
    // Hide all tabs
    document.querySelectorAll('.wallet-tab').forEach(tab => {
        tab.style.display = 'none';
    });
    
    // Show selected tab
    document.getElementById(tabName).style.display = 'block';
    
    // Update active tab button
    document.querySelectorAll('.wallet-tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[onclick="showWalletTab('${tabName}')"]`).classList.add('active');
}

// Load wallet addresses
async function loadWalletAddresses() {
    try {
        if (isTauri) {
            const result = await window.__TAURI__.invoke('get_wallet_info');
        if (result.success) {
                displayAddresses(result.addresses);
            }
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/addresses');
            const addresses = await response.json();
            displayAddresses(addresses);
        }
    } catch (error) {
        console.error('Failed to load wallet addresses:', error);
    }
}

// Display addresses
function displayAddresses(addresses) {
    const container = document.getElementById('addressesContainer');
    if (!container) return;
    
    container.innerHTML = Object.entries(addresses).map(([currency, address]) => `
        <div class="address-item">
            <span class="currency">${currency}</span>
            <span class="address" id="address-${currency}">${address}</span>
            <button onclick="copyAddress('address-${currency}')" class="copy-btn">Copy</button>
        </div>
    `).join('');
}

// Copy address to clipboard
async function copyAddress(elementId) {
    const element = document.getElementById(elementId);
    const address = element.textContent;
    
    try {
        if (isTauri) {
            await window.__TAURI__.clipboard.writeText(address);
        } else {
        await navigator.clipboard.writeText(address);
        }
        showNotification('Address copied to clipboard!', 'success');
    } catch (error) {
        console.error('Failed to copy address:', error);
        showNotification('Failed to copy address', 'error');
    }
}

// Send funds
async function sendFunds() {
    const toAddress = document.getElementById('sendToAddress').value;
    const amount = parseFloat(document.getElementById('sendAmount').value);
    const currency = document.getElementById('sendCurrency').value;
    
    if (!toAddress || !amount || amount <= 0) {
        showNotification('Please enter a valid address and amount', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            const result = await window.__TAURI__.invoke('send_funds', {
                toAddress,
                amount: Math.floor(amount * Math.pow(10, getCurrencyDecimals(currency))),
                currency
            });
            
            if (result.success) {
                showNotification(`Transaction sent! ID: ${result.transaction_id}`, 'success');
                
                // Clear form
                document.getElementById('sendToAddress').value = '';
                document.getElementById('sendAmount').value = '';
                document.getElementById('sendCurrency').value = 'USDC';
                
                // Refresh balances
                refreshBalances();
            } else {
                showNotification(result.message, 'error');
            }
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/send', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                to_address: toAddress,
                amount: Math.floor(amount * Math.pow(10, getCurrencyDecimals(currency))),
                    currency
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
            showNotification(`Transaction sent! ID: ${result.transaction_id}`, 'success');
            
            // Clear form
            document.getElementById('sendToAddress').value = '';
            document.getElementById('sendAmount').value = '';
                document.getElementById('sendCurrency').value = 'USDC';
            
            // Refresh balances
            refreshBalances();
        } else {
            showNotification(result.message, 'error');
            }
        }
        
    } catch (error) {
        console.error('Failed to send funds:', error);
        showNotification('Failed to send funds', 'error');
    }
}

// Refresh transaction history
async function refreshTransactionHistory() {
    try {
        if (isTauri) {
            showNotification('Transaction history will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/transactions');
            const transactions = await response.json();
            displayTransactionHistory(transactions);
        }
    } catch (error) {
        console.error('Failed to refresh transaction history:', error);
    }
}

// Display transaction history
function displayTransactionHistory(transactions) {
    const container = document.getElementById('transactionsContainer');
    if (!container) return;
    
    if (transactions.length === 0) {
        container.innerHTML = '<div class="transaction-item"><p>No transactions found</p></div>';
        return;
    }
    
    container.innerHTML = transactions.map(tx => `
        <div class="transaction-item">
            <div class="tx-header">
                <span class="tx-id">${tx.id}</span>
                <span class="tx-status ${tx.status}">${tx.status}</span>
                </div>
            <div class="tx-details">
                <p><strong>From:</strong> ${tx.from_address}</p>
                <p><strong>To:</strong> ${tx.to_address}</p>
                <p><strong>Amount:</strong> ${formatCurrencyAmount(tx.amount, tx.currency)} ${tx.currency}</p>
                <p><strong>Fee:</strong> ${formatCurrencyAmount(tx.fee, tx.currency)} ${tx.currency}</p>
                <p><strong>Timestamp:</strong> ${new Date(tx.timestamp * 1000).toLocaleString()}</p>
            </div>
        </div>
    `).join('');
}

// Filter transactions
function filterTransactions() {
    const filter = document.getElementById('transactionFilter').value;
    const transactions = document.querySelectorAll('.transaction-item');
    
    transactions.forEach(tx => {
        const status = tx.querySelector('.tx-status').textContent;
        if (filter === 'all' || status === filter) {
            tx.style.display = 'block';
        } else {
            tx.style.display = 'none';
        }
    });
}

// Refresh wallet keys
async function refreshKeys() {
    try {
        if (isTauri) {
            showNotification('Wallet keys will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/keys');
            const keys = await response.json();
            displayKeys(keys);
        }
    } catch (error) {
        console.error('Failed to refresh keys:', error);
    }
}

// Display keys
function displayKeys(keys) {
    const container = document.getElementById('keysContainer');
    if (!container) return;
    
    container.innerHTML = Object.entries(keys).map(([currency, key]) => `
        <div class="key-item">
            <span class="currency">${currency}</span>
            <span class="key" id="key-${currency}">${key}</span>
            <button onclick="copyKey('key-${currency}')" class="copy-btn">Copy</button>
        </div>
    `).join('');
}

// Copy key to clipboard
async function copyKey(elementId) {
    const element = document.getElementById(elementId);
    const key = element.textContent;
    
    try {
        if (isTauri) {
            await window.__TAURI__.clipboard.writeText(key);
        } else {
        await navigator.clipboard.writeText(key);
        }
        showNotification('Key copied to clipboard!', 'success');
    } catch (error) {
        console.error('Failed to copy key:', error);
        showNotification('Failed to copy key', 'error');
    }
}

// Backup wallet
async function backupWallet() {
    try {
        if (isTauri) {
            showNotification('Wallet backup will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/backup');
            const backup = await response.json();
            displayBackup(backup);
        }
    } catch (error) {
        console.error('Failed to backup wallet:', error);
        showNotification('Failed to backup wallet', 'error');
    }
}

// Display backup
function displayBackup(backup) {
    const container = document.getElementById('backupContainer');
    if (!container) return;
    
    container.innerHTML = `
        <div class="backup-item">
            <p><strong>Backup Data:</strong></p>
            <textarea id="backupData" readonly>${JSON.stringify(backup, null, 2)}</textarea>
            <button onclick="copyBackup()" class="copy-btn">Copy Backup</button>
        </div>
    `;
}

// Copy backup to clipboard
async function copyBackup() {
    const backupData = document.getElementById('backupData').value;
    
    try {
        if (isTauri) {
            await window.__TAURI__.clipboard.writeText(backupData);
        } else {
            await navigator.clipboard.writeText(backupData);
        }
        showNotification('Backup copied to clipboard!', 'success');
    } catch (error) {
        console.error('Failed to copy backup:', error);
        showNotification('Failed to copy backup', 'error');
    }
}

// Restore wallet
async function restoreWallet() {
    const backupData = document.getElementById('restoreData').value;
    
    if (!backupData) {
        showNotification('Please enter backup data', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            showNotification('Wallet restore will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/wallet/restore', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                backup_data: backupData
            })
        });
        
        const result = await response.json();
        
        if (result.success) {
                showNotification('Wallet restored successfully!', 'success');
            refreshBalances();
            loadWalletAddresses();
        } else {
            showNotification(result.message, 'error');
            }
        }
    } catch (error) {
        console.error('Failed to restore wallet:', error);
        showNotification('Failed to restore wallet', 'error');
    }
} 

// Register AOI key
async function registerAOIKey() {
    const key = document.getElementById('aoiKey').value;
    
    if (!key) {
        showNotification('Please enter an AOI key', 'error');
        return;
    }
    
    try {
        if (isTauri) {
            showNotification('AOI key registration will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/services/aoi/register', {
            method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    key
                })
        });
            
        const result = await response.json();
            
        if (result.success) {
            showNotification('AOI key registered successfully!', 'success');
                document.getElementById('aoiKey').value = '';
        } else {
            showNotification(result.message, 'error');
            }
        }
    } catch (error) {
        console.error('Failed to register AOI key:', error);
        showNotification('Failed to register AOI key', 'error');
    }
}

// Get AOI key
async function getAOIKey() {
    try {
        if (isTauri) {
            showNotification('AOI key retrieval will be available in a future update', 'info');
        } else {
            const response = await fetch('http://localhost:8081/api/services/aoi/get', {
            method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({})
        });
            
        const result = await response.json();
            
            if (result.success) {
                document.getElementById('aoiKeyDisplay').textContent = result.key;
            showNotification('AOI key retrieved successfully!', 'success');
        } else {
            showNotification(result.message, 'error');
            }
        }
    } catch (error) {
        console.error('Failed to get AOI key:', error);
        showNotification('Failed to get AOI key', 'error');
    }
} 