mod config;
mod db;
mod routes;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, put},
};
use config::Config;
use db::pool::create_pool;

use routes::brands::{create_brand, delete_brand, list_brands, update_brand};
use routes::business_profile::{get_business_profile, save_business_profile};
use routes::compositions::{
    create_composition, delete_composition, list_compositions, update_composition,
};
use routes::health::health_check;
use routes::inventory;
use routes::manufacturers::list_manufacturers;
use routes::parties;
use routes::products::{
    create_product, delete_product, get_product, list_products, update_product,
};
use routes::purchases;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

#[tokio::main]
async fn main() -> Result<()> {
    // --------------------------------------------------
    // LOGGING
    // --------------------------------------------------
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // --------------------------------------------------
    // CONFIGURATION
    // --------------------------------------------------
    let config = Config::from_env()?;

    // --------------------------------------------------
    // DATABASE
    // --------------------------------------------------
    let pool = create_pool(&config).await?;

    // --------------------------------------------------
    // CORS
    // --------------------------------------------------
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // --------------------------------------------------
    // API ROUTER
    // --------------------------------------------------
    let app = Router::new()
        // HEALTH
        .route("/api/health", get(health_check))
        // BUSINESS PROFILE
        .route(
            "/api/business-profile",
            get(get_business_profile).put(save_business_profile),
        )
        // PRODUCTS
        .route("/api/products", get(list_products).post(create_product))
        .route(
            "/api/products/{id}",
            get(get_product).put(update_product).delete(delete_product),
        )
        // BRANDS
        .route("/api/brands", get(list_brands).post(create_brand))
        .route("/api/brands/{id}", put(update_brand).delete(delete_brand))
        // MANUFACTURERS
        .route("/api/manufacturers", get(list_manufacturers))
        // COMPOSITIONS
        .route(
            "/api/compositions",
            get(list_compositions).post(create_composition),
        )
        .route(
            "/api/compositions/{id}",
            put(update_composition).delete(delete_composition),
        )
        // PARTIES
        .nest("/api/parties", parties::router())
        // HSN MASTER
        .nest("/api/hsn", routes::hsn::router())
        // INVENTORY
        .nest("/api/inventory", inventory::router())
        // PURCHASES
        .nest("/api/purchases", purchases::router())
        // SHARED APPLICATION STATE
        .with_state(pool)
        // MIDDLEWARE
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // --------------------------------------------------
    // SERVER ADDRESS
    // --------------------------------------------------
    let address = SocketAddr::new(config.server_host.parse()?, config.server_port);

    // --------------------------------------------------
    // TCP LISTENER
    // --------------------------------------------------
    let listener = TcpListener::bind(address).await?;

    // --------------------------------------------------
    // STARTUP INFORMATION
    // --------------------------------------------------
    println!();
    println!("================================================");
    println!("               ADHIRAJPOS API");
    println!("================================================");
    println!("Server:              http://{address}");
    println!("Health:              http://{address}/api/health");

    println!();
    println!("--- BUSINESS PROFILE ---");
    println!("GET /api/business-profile");
    println!("PUT /api/business-profile");

    println!();
    println!("--- PRODUCTS ---");
    println!("GET    /api/products");
    println!("POST   /api/products");
    println!("GET    /api/products/{{id}}");
    println!("PUT    /api/products/{{id}}");
    println!("DELETE /api/products/{{id}}");

    println!();
    println!("--- BRANDS ---");
    println!("GET    /api/brands");
    println!("POST   /api/brands");
    println!("PUT    /api/brands/{{id}}");
    println!("DELETE /api/brands/{{id}}");

    println!();
    println!("--- MANUFACTURERS ---");
    println!("GET /api/manufacturers");

    println!();
    println!("--- COMPOSITIONS ---");
    println!("GET    /api/compositions");
    println!("POST   /api/compositions");
    println!("PUT    /api/compositions/{{id}}");
    println!("DELETE /api/compositions/{{id}}");

    println!();
    println!("--- PARTIES ---");
    println!("GET  /api/parties");
    println!("POST /api/parties");
    println!("GET  /api/parties/{{id}}");
    println!("PUT  /api/parties/{{id}}");

    println!();
    println!("--- HSN MASTER ---");
    println!("GET /api/hsn");
    println!("GET /api/hsn/{{code}}");
    println!("GET /api/hsn/{{code}}/status");

    println!();
    println!("--- INVENTORY ---");
    println!("GET  /api/inventory/summary");
    println!("GET  /api/inventory/batches");
    println!("POST /api/inventory/adjustments");

    println!();
    println!("--- PURCHASES ---");
    println!("GET    /api/purchases");
    println!("POST   /api/purchases");
    println!("GET    /api/purchases/{{id}}");
    println!("PUT    /api/purchases/{{id}}");
    println!("DELETE /api/purchases/{{id}}");
    println!("POST   /api/purchases/{{id}} (POST PURCHASE)");

    println!();
    println!("================================================");
    println!("API SERVER READY");
    println!("================================================");
    println!();

    // --------------------------------------------------
    // START SERVER
    // --------------------------------------------------
    axum::serve(listener, app).await?;

    Ok(())
}
