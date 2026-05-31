//! `CartReactor` — shopping cart with multiple state fields.
//!
//! Demonstrates a cubit whose single state struct holds several pieces of
//! data that all evolve together:
//!
//! ```text
//! CartState {
//!     items:    Vec<CartItem>  — the items in the cart
//!     total:    f64            — computed price after discount
//!     discount: f64            — discount percentage (0.0 – 1.0)
//!     status:   CartStatus     — Empty | Active | CheckedOut
//! }
//! ```
//!
//! Every method on the cubit computes a completely new `CartState` and
//! calls `emit()` — state is never mutated in place.

use gloc::reactor;
use gloc::Reactor;

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// A single product in the cart.
#[derive(Clone, PartialEq, Debug)]
pub struct CartItem {
    pub name: String,
    pub price: f64,
}

/// The lifecycle phase of the cart.
#[derive(Clone, PartialEq, Debug)]
pub enum CartStatus {
    /// Cart has no items.
    Empty,
    /// Cart has items and is being built.
    Active,
    /// Order has been placed — cart is locked.
    CheckedOut,
}

impl CartStatus {
    pub fn label(&self) -> &'static str {
        match self {
            CartStatus::Empty => "Empty",
            CartStatus::Active => "Active",
            CartStatus::CheckedOut => "Checked Out",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            CartStatus::Empty => "#6b7280",
            CartStatus::Active => "#2563eb",
            CartStatus::CheckedOut => "#16a34a",
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// A snapshot of the cart at a single point in time.
///
/// All fields are derived from the cart's items and discount — the cubit
/// computes them on every transition so the UI never needs to.
#[derive(Clone, PartialEq, Debug)]
pub struct CartState {
    /// Items currently in the cart.
    pub items: Vec<CartItem>,
    /// Raw total before discount.
    pub subtotal: f64,
    /// Discount percentage applied (0.0 = none, 0.2 = 20% off).
    pub discount: f64,
    /// Final total after discount.
    pub total: f64,
    /// Lifecycle phase of the cart.
    pub status: CartStatus,
}

impl CartState {
    /// Constructs an empty initial state.
    pub fn empty() -> Self {
        Self {
            items: vec![],
            subtotal: 0.0,
            discount: 0.0,
            total: 0.0,
            status: CartStatus::Empty,
        }
    }

    /// Rebuilds all computed fields from `items` and `discount`.
    ///
    /// Called internally by the cubit after every mutation so every
    /// field in the state is always consistent.
    fn recompute(items: Vec<CartItem>, discount: f64) -> Self {
        let subtotal = items.iter().map(|i| i.price).sum::<f64>();
        let total = subtotal * (1.0 - discount);
        let status = if items.is_empty() {
            CartStatus::Empty
        } else {
            CartStatus::Active
        };
        Self {
            items,
            subtotal,
            discount,
            total,
            status,
        }
    }
}

// ---------------------------------------------------------------------------
// Cubit
// ---------------------------------------------------------------------------

/// Manages the shopping cart state.
///
/// A single cubit owns a state with multiple fields — items, totals,
/// discount, and status all live in one `CartState` snapshot.
///
/// The macro generates `new()`, `impl Reactor`, and `on_change()`.
#[reactor(state = CartState)]
pub struct CartReactor {}

impl CartReactor {
    /// Adds an item to the cart and recomputes all totals.
    ///
    /// # Parameters
    /// - `name`  — product name
    /// - `price` — product price
    pub fn add_item(&mut self, name: &str, price: f64) {
        if matches!(self.state().status, CartStatus::CheckedOut) {
            return; // locked after checkout
        }
        let mut items = self.state().items.clone();
        items.push(CartItem {
            name: name.to_string(),
            price,
        });
        self.emit(CartState::recompute(items, self.state().discount));
    }

    /// Removes the item at `index` from the cart.
    ///
    /// # Parameters
    /// - `index` — position of the item to remove
    pub fn remove_item(&mut self, index: usize) {
        if matches!(self.state().status, CartStatus::CheckedOut) {
            return;
        }
        let mut items = self.state().items.clone();
        if index < items.len() {
            items.remove(index);
            self.emit(CartState::recompute(items, self.state().discount));
        }
    }

    /// Applies a discount percentage to the cart.
    ///
    /// # Parameters
    /// - `pct` — discount as a fraction between 0.0 and 1.0, e.g. `0.1` = 10% off
    pub fn apply_discount(&mut self, pct: f64) {
        if matches!(self.state().status, CartStatus::CheckedOut) {
            return;
        }
        let pct = pct.clamp(0.0, 1.0);
        self.emit(CartState::recompute(self.state().items.clone(), pct));
    }

    /// Removes any applied discount.
    #[allow(dead_code)]
    pub fn remove_discount(&mut self) {
        self.apply_discount(0.0);
    }

    /// Places the order — transitions cart to `CheckedOut`.
    ///
    /// The cart is locked after this; no further items can be added or removed.
    pub fn checkout(&mut self) {
        if !matches!(self.state().status, CartStatus::Active) {
            return;
        }
        let mut next = self.state().clone();
        next.status = CartStatus::CheckedOut;
        self.emit(next);
    }

    /// Clears the cart and resets to empty.
    pub fn clear(&mut self) {
        self.emit(CartState::empty());
    }
}
