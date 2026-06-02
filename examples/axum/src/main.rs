//! # GLoC Shop API — Axum Example
//!
//! A realistic multi-reactor Axum application that demonstrates GLoC features
//! in a web API context.
//!
//! ## Features demonstrated
//!
//! | Pattern | Where |
//! |---------|-------|
//! | `#[reactor(state = T)]` Mode A | `CartReactor`, `InventoryReactor` |
//! | `neutrons = E` — neutron firing | `CartReactor` + `CartEvent` |
//! | `reactor.fire(neutron)` entry-point | Cart route handlers |
//! | `AxumReactor<R>` shared state | `AppState` struct |
//! | Multi-reactor `AppState` | `Router::with_state(app_state)` |
//! | Change-detection (no-op guard) | Checkout on already-checked-out cart |
//!
//! ## Routes
//!
//! ```text
//! GET  /cart                   — full cart state as JSON
//! POST /cart/items             — { "name": "...", "price": 0.0 }
//! DELETE /cart/items/:index    — remove item at index
//! POST /cart/discount          — { "percent": 0.15 }
//! POST /cart/checkout          — lock the cart
//! DELETE /cart                 — clear the cart
//!
//! GET  /inventory              — full inventory as JSON
//! POST /inventory/restock      — { "name": "...", "qty": 10 }
//! POST /inventory/deduct       — { "name": "...", "qty": 1 }
//! ```
//!
//! ## Quick start
//!
//! ```sh
//! cargo run -p gloc-example-axum
//! # then:
//! curl http://localhost:3000/cart
//! curl -X POST http://localhost:3000/cart/items \
//!      -H 'Content-Type: application/json' \
//!      -d '{"name":"Widget","price":9.99}'
//! curl -X POST http://localhost:3000/cart/checkout
//! curl http://localhost:3000/inventory
//! curl -X POST http://localhost:3000/inventory/restock \
//!      -H 'Content-Type: application/json' \
//!      -d '{"name":"Widget","qty":100}'
//! ```

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use gloc::{reactor, reactor_state, Reactor};
use gloc_axum::AxumReactor;
use serde::{Deserialize, Serialize};

// ============================================================================
// Cart domain
// ============================================================================

/// A single line-item in the shopping cart.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CartItem {
    /// Display name of the product.
    pub name: String,
    /// Unit price in the store's currency.
    pub price: f64,
}

/// Lifecycle state of the cart — prevents mutations after checkout.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum CartStatus {
    /// Cart is accepting items and modifications.
    #[default]
    Open,
    /// Cart has been checked out; mutations are rejected until cleared.
    CheckedOut,
}

/// Complete snapshot of the shopping cart at a single point in time.
///
/// All numeric totals are derived values recalculated on every mutation,
/// so the API never returns stale subtotals.
#[reactor_state(derive(Serialize))]
pub struct CartState {
    /// Ordered list of items added to the cart.
    pub items: Vec<CartItem>,
    /// Sum of all item prices before any discount.
    pub subtotal: f64,
    /// Discount amount already subtracted from the subtotal (in currency units).
    pub discount: f64,
    /// Final amount due: `subtotal - discount`.
    pub total: f64,
    /// Whether the cart is open for editing or locked after checkout.
    pub status: CartStatus,
}

impl CartState {
    /// Constructs an empty, open cart ready to accept items.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            subtotal: 0.0,
            discount: 0.0,
            total: 0.0,
            status: CartStatus::Open,
        }
    }
}

/// Every command that can be sent to `CartReactor` via the event bus.
///
/// Using an event enum alongside direct methods shows that both styles
/// coexist on the same reactor — the API layer can choose whichever is
/// clearest for each route handler.
#[derive(Debug)]
pub enum CartEvent {
    /// Add an item with the given name and price.
    AddItem { name: String, price: f64 },
    /// Remove the item at the given zero-based index.
    RemoveItem(usize),
    /// Apply a fractional discount (e.g. `0.15` for 15%).
    ApplyDiscount(f64),
    /// Lock the cart — no further modifications allowed until cleared.
    Checkout,
    /// Reset the cart to its empty initial state.
    Clear,
}

/// Manages the shopping cart: items, totals, discounts, and checkout status.
///
/// `#[reactor(state = CartState, neutrons = CartEvent)]` generates:
/// - `impl Reactor for CartReactor` — `state()`, `emit()` with change-detection
/// - `pub fn new(initial: CartState) -> Self`
/// - `pub fn fire(&mut self, neutron: CartEvent)` — calls `self.on_event(neutron)`
/// - `subscribe()`, `attach_listener()`
///
/// Route handlers call `consumer.update(|r| r.fire(event))` or
/// `consumer.update(|r| r.add_item(name, price))` interchangeably.
#[reactor(state = CartState, neutrons = CartEvent)]
pub struct CartReactor {}

impl CartReactor {
    // -- Direct methods -------------------------------------------------------

    /// Appends an item and recalculates subtotal and total.
    ///
    /// No-ops if the cart is already checked out — the method returns early
    /// without calling `emit`, preserving change-detection semantics.
    pub fn add_item(&mut self, name: String, price: f64) {
        if self.state().status == CartStatus::CheckedOut {
            println!("[CartReactor] Rejected: cart is checked out");
            return;
        }
        let mut next = self.state().clone();
        next.items.push(CartItem { name, price });
        Self::recalculate(&mut next);
        self.emit(next);
    }

    /// Removes the item at `index` if it exists.
    ///
    /// Out-of-bounds indices are silently ignored so callers do not need to
    /// validate the index before calling.
    pub fn remove_item(&mut self, index: usize) {
        if self.state().status == CartStatus::CheckedOut {
            println!("[CartReactor] Rejected: cart is checked out");
            return;
        }
        let mut next = self.state().clone();
        if index < next.items.len() {
            next.items.remove(index);
            Self::recalculate(&mut next);
            self.emit(next);
        } else {
            println!("[CartReactor] remove_item: index {} out of range", index);
        }
    }

    /// Applies a fractional discount percentage, clamped to [0.0, 1.0].
    ///
    /// Recalculates `discount` and `total` but leaves the item list unchanged.
    pub fn apply_discount(&mut self, percent: f64) {
        if self.state().status == CartStatus::CheckedOut {
            println!("[CartReactor] Rejected: cart is checked out");
            return;
        }
        let pct = percent.clamp(0.0, 1.0);
        let mut next = self.state().clone();
        next.discount = (next.subtotal * pct * 100.0).round() / 100.0;
        next.total = (next.subtotal - next.discount).max(0.0);
        self.emit(next);
    }

    /// Locks the cart — subsequent add/remove/discount calls are rejected.
    ///
    /// Calling checkout on an already-checked-out cart emits the same state,
    /// which GLoC's change-detection discards as a no-op.
    pub fn checkout(&mut self) {
        let mut next = self.state().clone();
        next.status = CartStatus::CheckedOut;
        self.emit(next);
    }

    /// Resets the cart to an empty, open state — discarding all items.
    ///
    /// This is the only way to re-open a checked-out cart.
    pub fn clear(&mut self) {
        self.emit(CartState::empty());
    }

    // -- on_event — required when `neutrons = CartEvent` is used ----------------

    /// Routes each `CartEvent` variant to the corresponding direct method.
    ///
    /// The macro generates `pub fn fire(neutron: CartEvent)` which calls here.
    /// The user owns this implementation — the macro only generates the public
    /// entry-point wrapper.
    fn on_event(&mut self, event: CartEvent) {
        match event {
            CartEvent::AddItem { name, price } => self.add_item(name, price),
            CartEvent::RemoveItem(index) => self.remove_item(index),
            CartEvent::ApplyDiscount(pct) => self.apply_discount(pct),
            CartEvent::Checkout => self.checkout(),
            CartEvent::Clear => self.clear(),
        }
    }

    // -- Helpers --------------------------------------------------------------

    /// Recomputes `subtotal` and `total` from the current item list.
    ///
    /// Called by every mutation that changes items so totals are always
    /// derived and never drift from the item list.
    fn recalculate(state: &mut CartState) {
        let subtotal: f64 = state.items.iter().map(|i| i.price).sum();
        state.subtotal = (subtotal * 100.0).round() / 100.0;
        state.total = ((subtotal - state.discount).max(0.0) * 100.0).round() / 100.0;
    }
}

// ============================================================================
// Inventory domain
// ============================================================================

/// Complete snapshot of the product inventory at a single point in time.
///
/// Uses a `HashMap` keyed by product name so stock queries and updates
/// are O(1) regardless of catalogue size.
#[reactor_state(derive(Serialize))]
pub struct InventoryState {
    /// Map from product name to current stock count.
    pub items: HashMap<String, u32>,
}

impl InventoryState {
    /// Constructs an empty inventory with no products.
    pub fn empty() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}

/// Tracks stock levels for all products and enforces a non-negative invariant.
///
/// `#[reactor(state = InventoryState)]` generates the full `Reactor` impl,
/// constructor, and `subscribe` / `attach_listener` observer API.
///
/// Stock deductions are no-ops when the product is out of stock, which keeps
/// the invariant that `items[name] >= 0` without requiring `i64`.
#[reactor(state = InventoryState)]
pub struct InventoryReactor {}

impl InventoryReactor {
    /// Adds `qty` units to `name`, creating the entry if it does not yet exist.
    ///
    /// Restocking an unknown product bootstraps it at `qty` so callers do not
    /// need a separate "create product" step.
    pub fn restock(&mut self, name: String, qty: u32) {
        let mut next = self.state().clone();
        let entry = next.items.entry(name.clone()).or_insert(0);
        *entry += qty;
        println!(
            "[InventoryReactor] Restocked {}: +{} (now {})",
            name, qty, *entry
        );
        self.emit(next);
    }

    /// Deducts `qty` units from `name`, clamped at zero.
    ///
    /// If `name` is not in the inventory or stock is already zero, the call is
    /// silently ignored — callers can check `/inventory` to confirm availability
    /// before dispatching.
    pub fn deduct(&mut self, name: String, qty: u32) {
        let mut next = self.state().clone();
        match next.items.get_mut(&name) {
            Some(stock) if *stock > 0 => {
                *stock = stock.saturating_sub(qty);
                println!(
                    "[InventoryReactor] Deducted {}: -{} (now {})",
                    name, qty, *stock
                );
                self.emit(next);
            }
            Some(_) => println!("[InventoryReactor] {} is out of stock", name),
            None => println!("[InventoryReactor] Unknown product: {}", name),
        }
    }
}

// ============================================================================
// AppState — both reactors in one cloneable handle
// ============================================================================

/// Shared application state injected into every Axum route handler.
///
/// Cloning is free — each field is an `AxumReactor<R>` which wraps
/// `Arc`-reference-counted internals. Axum clones this once per request.
#[derive(Clone)]
pub struct AppState {
    /// The shopping cart reactor — shared across all cart route handlers.
    pub cart: AxumReactor<CartReactor>,
    /// The inventory reactor — shared across all inventory route handlers.
    pub inventory: AxumReactor<InventoryReactor>,
}

// ============================================================================
// Request / response types
// ============================================================================

/// Request body for `POST /cart/items`.
#[derive(Deserialize)]
pub struct AddItemRequest {
    /// Product display name.
    pub name: String,
    /// Unit price in store currency.
    pub price: f64,
}

/// Request body for `POST /cart/discount`.
#[derive(Deserialize)]
pub struct DiscountRequest {
    /// Fractional discount percentage (e.g. `0.10` for 10%).
    pub percent: f64,
}

/// Request body for `POST /inventory/restock`.
#[derive(Deserialize)]
pub struct RestockRequest {
    /// Product name to restock.
    pub name: String,
    /// Number of units to add.
    pub qty: u32,
}

/// Request body for `POST /inventory/deduct`.
#[derive(Deserialize)]
pub struct DeductRequest {
    /// Product name to deduct stock from.
    pub name: String,
    /// Number of units to remove.
    pub qty: u32,
}

// ============================================================================
// Cart route handlers
// ============================================================================

/// Returns the full cart state as JSON.
///
/// Always succeeds — an empty cart is a valid, serialisable state.
async fn get_cart(State(app): State<AppState>) -> impl IntoResponse {
    Json(app.cart.state())
}

/// Adds an item to the cart.
///
/// Rejected with `409 Conflict` if the cart has already been checked out.
async fn add_cart_item(
    State(app): State<AppState>,
    Json(body): Json<AddItemRequest>,
) -> impl IntoResponse {
    let was_checked_out = app.cart.state().status == CartStatus::CheckedOut;
    app.cart.update(|r| {
        r.fire(CartEvent::AddItem {
            name: body.name,
            price: body.price,
        })
    });
    if was_checked_out {
        StatusCode::CONFLICT.into_response()
    } else {
        Json(app.cart.state()).into_response()
    }
}

/// Removes the item at the given zero-based index.
///
/// Returns `404 Not Found` when the index is out of range.
async fn remove_cart_item(
    State(app): State<AppState>,
    Path(index): Path<usize>,
) -> impl IntoResponse {
    let len = app.cart.state().items.len();
    if index >= len {
        return StatusCode::NOT_FOUND.into_response();
    }
    app.cart.update(|r| r.fire(CartEvent::RemoveItem(index)));
    Json(app.cart.state()).into_response()
}

/// Applies a fractional discount to the cart total.
///
/// The percent value is clamped to [0.0, 1.0] inside the reactor.
async fn apply_cart_discount(
    State(app): State<AppState>,
    Json(body): Json<DiscountRequest>,
) -> impl IntoResponse {
    app.cart
        .update(|r| r.fire(CartEvent::ApplyDiscount(body.percent)));
    Json(app.cart.state())
}

/// Checks out the cart — no further modifications are accepted until cleared.
///
/// Idempotent: checking out an already-checked-out cart is a no-op.
async fn checkout_cart(State(app): State<AppState>) -> impl IntoResponse {
    app.cart.update(|r| r.fire(CartEvent::Checkout));
    Json(app.cart.state())
}

/// Clears all items and reopens the cart.
async fn clear_cart(State(app): State<AppState>) -> impl IntoResponse {
    app.cart.update(|r| r.fire(CartEvent::Clear));
    Json(app.cart.state())
}

// ============================================================================
// Inventory route handlers
// ============================================================================

/// Returns the full inventory as JSON.
async fn get_inventory(State(app): State<AppState>) -> impl IntoResponse {
    Json(app.inventory.state())
}

/// Adds stock units for a product — creates the product entry if absent.
async fn restock_item(
    State(app): State<AppState>,
    Json(body): Json<RestockRequest>,
) -> impl IntoResponse {
    app.inventory.update(|r| r.restock(body.name, body.qty));
    Json(app.inventory.state())
}

/// Deducts stock units for a product.
///
/// Returns `404 Not Found` when the product is unknown or has no stock.
async fn deduct_item(
    State(app): State<AppState>,
    Json(body): Json<DeductRequest>,
) -> impl IntoResponse {
    let available = app
        .inventory
        .state()
        .items
        .get(&body.name)
        .copied()
        .unwrap_or(0);

    if available == 0 {
        return StatusCode::NOT_FOUND.into_response();
    }
    app.inventory.update(|r| r.deduct(body.name, body.qty));
    Json(app.inventory.state()).into_response()
}

// ============================================================================
// Entry point
// ============================================================================

#[tokio::main]
async fn main() {
    let cart = CartReactor::new(CartState::empty());
    let inventory = InventoryReactor::new(InventoryState::empty());

    // -- Wrap in AxumReactor and compose into AppState ------------------------
    let app_state = AppState {
        cart: AxumReactor::new(cart),
        inventory: AxumReactor::new(inventory),
    };

    // -- Build the router -----------------------------------------------------
    let app = Router::new()
        // Cart routes
        .route("/cart", get(get_cart))
        .route("/cart/items", post(add_cart_item))
        .route("/cart/items/:index", delete(remove_cart_item))
        .route("/cart/discount", post(apply_cart_discount))
        .route("/cart/checkout", post(checkout_cart))
        .route("/cart", delete(clear_cart))
        // Inventory routes
        .route("/inventory", get(get_inventory))
        .route("/inventory/restock", post(restock_item))
        .route("/inventory/deduct", post(deduct_item))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("GLoC Shop API listening on http://localhost:3000");
    println!("Try:");
    println!("  curl http://localhost:3000/cart");
    println!("  curl -X POST http://localhost:3000/cart/items \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\":\"Widget\",\"price\":9.99}}'");
    println!("  curl -X POST http://localhost:3000/cart/checkout");
    println!("  curl http://localhost:3000/inventory");
    println!("  curl -X POST http://localhost:3000/inventory/restock \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\":\"Widget\",\"qty\":100}}'");

    axum::serve(listener, app).await.expect("server error");
}
