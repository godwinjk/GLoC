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

impl Default for CartState {
    fn default() -> Self {
        Self {
            items: vec![],
            subtotal: 0.0,
            discount: 0.0,
            total: 0.0,
            status: CartStatus::Empty,
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
/// The macro generates `new()` and `impl Reactor`.
#[reactor(state = CartState)]
pub struct CartReactor {}

impl CartReactor {
    /// Adds an item to the cart and recomputes all totals.
    pub fn add_item(&mut self, name: &str, price: f64) {
        if matches!(self.state().status, CartStatus::CheckedOut) {
            return;
        }
        let mut items = self.state().items.clone();
        items.push(CartItem {
            name: name.to_string(),
            price,
        });
        self.emit(self.recompute(items, self.state().discount));
    }

    /// Removes the item at `index` from the cart.
    pub fn remove_item(&mut self, index: usize) {
        if matches!(self.state().status, CartStatus::CheckedOut) {
            return;
        }
        let mut items = self.state().items.clone();
        if index < items.len() {
            items.remove(index);
            self.emit(self.recompute(items, self.state().discount));
        }
    }

    /// Applies a discount percentage (0.0–1.0) to the cart.
    pub fn apply_discount(&mut self, pct: f64) {
        if matches!(self.state().status, CartStatus::CheckedOut) {
            return;
        }
        self.emit(self.recompute(self.state().items.clone(), pct.clamp(0.0, 1.0)));
    }

    /// Removes any applied discount.
    #[allow(dead_code)]
    pub fn remove_discount(&mut self) {
        self.apply_discount(0.0);
    }

    /// Places the order — transitions cart to `CheckedOut` and locks it.
    pub fn checkout(&mut self) {
        if !matches!(self.state().status, CartStatus::Active) {
            return;
        }
        let mut next = self.state().clone();
        next.status = CartStatus::CheckedOut;
        self.emit(next);
    }

    /// Clears the cart and resets to the default empty state.
    pub fn clear(&mut self) {
        self.emit(CartState::default());
    }

    /// Derives a new `CartState` from raw `items` and `discount`.
    ///
    /// All computed fields (`subtotal`, `total`, `status`) are calculated here
    /// so the reactor — not the state — owns the derivation logic.
    fn recompute(&self, items: Vec<CartItem>, discount: f64) -> CartState {
        let subtotal = items.iter().map(|i| i.price).sum::<f64>();
        let total = subtotal * (1.0 - discount);
        let status = if items.is_empty() {
            CartStatus::Empty
        } else {
            CartStatus::Active
        };
        CartState {
            items,
            subtotal,
            discount,
            total,
            status,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gloc_test::{reactor_test, ReactorTester};

    use super::*;

    fn cart() -> CartReactor {
        CartReactor::new(CartState::default())
    }

    // ---- happy path ----

    #[test]
    fn add_item_transitions_status_to_active() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 12.99));
        assert_eq!(tester.state().status, CartStatus::Active);
    }

    #[test]
    fn add_item_updates_subtotal_and_total() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Pen", 1.50));
        tester.act(|r| r.add_item("Book", 10.00));
        let s = tester.state();
        assert!((s.subtotal - 11.50).abs() < 0.001);
        assert!((s.total - 11.50).abs() < 0.001);
    }

    #[test]
    fn remove_item_recalculates_total() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Pen", 1.50));
        tester.act(|r| r.add_item("Book", 10.00));
        tester.act(|r| r.remove_item(0)); // remove Pen
        let s = tester.state();
        assert_eq!(s.items.len(), 1);
        assert!((s.total - 10.00).abs() < 0.001);
    }

    #[test]
    fn apply_discount_reduces_total() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Bag", 20.00));
        tester.act(|r| r.apply_discount(0.25));
        let s = tester.state();
        assert!((s.total - 15.00).abs() < 0.001);
        assert!((s.discount - 0.25).abs() < 0.001);
    }

    #[test]
    fn checkout_transitions_status_to_checked_out() {
        reactor_test! {
            build: cart(),
            acts: [
                |r| r.add_item("Book", 12.99),
                |r| r.checkout(),
            ],
            expect_states: [
                CartState {
                    items: vec![CartItem { name: "Book".into(), price: 12.99 }],
                    subtotal: 12.99,
                    discount: 0.0,
                    total: 12.99,
                    status: CartStatus::Active,
                },
                CartState {
                    items: vec![CartItem { name: "Book".into(), price: 12.99 }],
                    subtotal: 12.99,
                    discount: 0.0,
                    total: 12.99,
                    status: CartStatus::CheckedOut,
                },
            ],
        }
    }

    #[test]
    fn clear_resets_cart_to_empty() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 12.99));
        tester.act(|r| r.clear());
        assert_eq!(tester.state(), CartState::default());
    }

    // ---- edge cases ----

    #[test]
    fn checkout_on_empty_cart_emits_nothing() {
        // Checkout guard: only Active carts can check out.
        reactor_test! {
            build: cart(),
            acts: [|r| r.checkout()],
            expect_no_emissions: true,
        }
    }

    #[test]
    fn add_item_after_checkout_is_ignored() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 12.99));
        tester.act(|r| r.checkout());
        let emissions_before = tester.emitted_states().len();

        tester.act(|r| r.add_item("Pen", 1.49));
        assert_eq!(tester.emitted_states().len(), emissions_before);
    }

    #[test]
    fn remove_item_after_checkout_is_ignored() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 12.99));
        tester.act(|r| r.checkout());
        let item_count = tester.state().items.len();

        tester.act(|r| r.remove_item(0));
        assert_eq!(tester.state().items.len(), item_count);
    }

    #[test]
    fn clear_on_empty_cart_emits_nothing() {
        // CartState::default() == CartState::default() — change-detection swallows it.
        reactor_test! {
            build: cart(),
            acts: [|r| r.clear()],
            expect_no_emissions: true,
        }
    }

    // ---- boundary ----

    #[test]
    fn remove_item_out_of_bounds_is_safe() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 12.99));
        let before = tester.state();

        tester.act(|r| r.remove_item(99)); // no such index
        assert_eq!(tester.state(), before);
    }

    #[test]
    fn discount_above_one_is_clamped_to_one() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 20.00));
        tester.act(|r| r.apply_discount(1.5)); // clamped to 1.0 → total = 0
        let s = tester.state();
        assert!((s.total - 0.0).abs() < 0.001);
        assert!((s.discount - 1.0).abs() < 0.001);
    }

    #[test]
    fn remove_discount_restores_full_price() {
        let tester = ReactorTester::new(cart());
        tester.act(|r| r.add_item("Book", 20.00));
        tester.act(|r| r.apply_discount(0.5));
        tester.act(|r| r.remove_discount());
        assert!((tester.state().total - 20.00).abs() < 0.001);
    }
}
