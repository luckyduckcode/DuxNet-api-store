//! # Marketplace API Module
//!
//! This module provides a GitHub-like marketplace interface for discovering and browsing
//! services in the DuxNet ecosystem. It includes advanced search, filtering, sorting,
//! and pagination capabilities.
//!
//! ## Features
//! - Advanced search with text queries
//! - Category and tag filtering
//! - Price and reputation filtering
//! - Multiple sort options (relevance, popularity, price, etc.)
//! - Pagination support
//! - Popular services discovery
//! - Service statistics and analytics

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::api::state::ApiState;
use crate::core::data_structures::*;
use crate::core::data_structures::ServiceManifest;
use tracing::{info, warn};

/// Search the marketplace with advanced filtering and sorting
pub async fn search_marketplace(
    State(state): State<ApiState>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    info!("Marketplace search: query='{}', category={:?}, tags={:?}", 
          params.query, params.category, params.tags);

    // Create search filters
    let filters = SearchFilters {
        category: params.category.clone(),
        tags: params.tags.clone(),
        min_reputation: params.filters.as_ref().and_then(|f| f.min_reputation),
        max_price: params.filters.as_ref().and_then(|f| f.max_price),
        author: params.filters.as_ref().and_then(|f| f.author.clone()),
    };

    // Search DHT for matching manifests
    let mut results = state.node.dht.search_manifests(&params.query, filters.clone()).await;

    // Apply sorting
    sort_services(&mut results, &params.sort_by);

    // Calculate pagination
    let total = results.len();
    let total_pages = ((total as f64) / (params.limit as f64)).ceil() as u32;
    let start = (params.offset as usize).min(total);
    let end = (start + params.limit as usize).min(total);
    let paginated_results = results[start..end].to_vec();

    let response = SearchResponse {
        services: paginated_results,
        total,
        query: params.query,
        filters_applied: Some(filters),
        sort_by: params.sort_by,
        page: (params.offset / params.limit) + 1,
        total_pages,
    };

    Json(response)
}

/// Get popular/trending services
pub async fn get_popular_services(
    State(state): State<ApiState>,
    Query(params): Query<PopularServicesRequest>,
) -> impl IntoResponse {
    info!("Getting popular services: limit={}", params.limit.unwrap_or(10));

    let limit = params.limit.unwrap_or(10).min(100) as usize; // Cap at 100
    let services = state.node.dht.get_popular_services(limit).await;
    let services_len = services.len();

    Json(PopularServicesResponse {
        services,
        total: services_len,
        limit: limit as u32,
    })
}

/// Get all available categories
pub async fn get_categories(State(state): State<ApiState>) -> impl IntoResponse {
    info!("Getting marketplace categories");

    let categories = state.node.dht.get_manifest_categories().await;
    let categories_len = categories.len();

    Json(CategoriesResponse {
        categories,
        total: categories_len,
    })
}

/// Get all available tags
pub async fn get_tags(State(state): State<ApiState>) -> impl IntoResponse {
    info!("Getting marketplace tags");

    let tags = state.node.dht.get_manifest_tags().await;
    let tags_len = tags.len();

    Json(TagsResponse {
        tags,
        total: tags_len,
    })
}

/// Get marketplace statistics
pub async fn get_marketplace_stats(State(state): State<ApiState>) -> impl IntoResponse {
    info!("Getting marketplace statistics");

    let dht_stats = state.node.dht.get_stats().await;
    let categories = state.node.dht.get_manifest_categories().await;
    let tags = state.node.dht.get_manifest_tags().await;

    Json(MarketplaceStats {
        total_services: dht_stats.manifest_entries,
        total_categories: categories.len(),
        total_tags: tags.len(),
        total_authors: get_unique_authors_count(&state).await,
        dht_stats,
    })
}

/// Get featured services (highly rated, popular, or promoted)
pub async fn get_featured_services(
    State(state): State<ApiState>,
    Query(params): Query<FeaturedServicesRequest>,
) -> impl IntoResponse {
    info!("Getting featured services");

    let limit = params.limit.unwrap_or(6).min(20) as usize;
    
    // Get services with high reputation scores
    let filters = SearchFilters {
        category: None,
        tags: vec![],
        min_reputation: Some(4.0), // Only high-rated services
        max_price: None,
        author: None,
    };

    let mut services = state.node.dht.search_manifests("", filters).await;
    
    // Sort by reputation score descending
    services.sort_by(|a, b| {
        let a_score = a.reputation.as_ref().map(|r| r.score).unwrap_or(0.0);
        let b_score = b.reputation.as_ref().map(|r| r.score).unwrap_or(0.0);
        b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
    });

    let featured = services.into_iter().take(limit).collect();

    Json(FeaturedServicesResponse {
        services: featured,
        total: limit,
    })
}

// === Helper Functions ===

/// Sort services based on the specified sort option
fn sort_services(services: &mut Vec<ServiceManifest>, sort_by: &SortOption) {
    match sort_by {
        SortOption::Relevance => {
            // Default order - could implement relevance scoring
        }
        SortOption::Newest => {
            services.sort_by(|a, b| {
                // Parse version as semver or use string comparison
                b.version.cmp(&a.version)
            });
        }
        SortOption::Oldest => {
            services.sort_by(|a, b| {
                a.version.cmp(&b.version)
            });
        }
        SortOption::MostPopular => {
            services.sort_by(|a, b| {
                let a_score = a.reputation.as_ref().map(|r| r.reviews).unwrap_or(0);
                let b_score = b.reputation.as_ref().map(|r| r.reviews).unwrap_or(0);
                b_score.cmp(&a_score)
            });
        }
        SortOption::HighestRated => {
            services.sort_by(|a, b| {
                let a_score = a.reputation.as_ref().map(|r| r.score).unwrap_or(0.0);
                let b_score = b.reputation.as_ref().map(|r| r.score).unwrap_or(0.0);
                b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        SortOption::LowestPrice => {
            // Skip price sorting since PricingModel is an enum
            // TODO: Implement price extraction based on pricing model
        }
        SortOption::HighestPrice => {
            // Skip price sorting since PricingModel is an enum  
            // TODO: Implement price extraction based on pricing model
        }
    }
}

/// Get count of unique authors in the marketplace
async fn get_unique_authors_count(state: &ApiState) -> usize {
    // This would be more efficient with a dedicated author index
    let filters = SearchFilters {
        category: None,
        tags: vec![],
        min_reputation: None,
        max_price: None,
        author: None,
    };

    let services = state.node.dht.search_manifests("", filters).await;
    let unique_authors: std::collections::HashSet<String> = services
        .iter()
        .map(|s| s.author.did.clone())
        .collect();

    unique_authors.len()
}

// === Request/Response Structures ===

#[derive(Debug, Deserialize)]
pub struct PopularServicesRequest {
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PopularServicesResponse {
    pub services: Vec<ServiceManifest>,
    pub total: usize,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct TagsResponse {
    pub tags: Vec<String>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceStats {
    pub total_services: usize,
    pub total_categories: usize,
    pub total_tags: usize,
    pub total_authors: usize,
    pub dht_stats: crate::core::dht::DHTStats,
}

#[derive(Debug, Deserialize)]
pub struct FeaturedServicesRequest {
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct FeaturedServicesResponse {
    pub services: Vec<ServiceManifest>,
    pub total: usize,
}
