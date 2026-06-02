package com.gloc.plugin

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test

class ReactorGeneratorTest {

    @Test
    fun `withoutNeutrons does not contain neutron references`() {
        val content = ReactorGenerator.generateContent("Foo", withNeutrons = false)
        assertTrue(content.contains("#[reactor(state = FooState)]"), "Must have reactor macro without neutrons")
        assertFalse(content.contains("neutrons"), "Must not reference neutrons")
        assertFalse(content.contains("FooNeutron"), "Must not reference FooNeutron")
        assertFalse(content.contains("on_event"), "Must not generate on_event")
    }

    @Test
    fun `withNeutrons generates neutron enum and on_event handler`() {
        val content = ReactorGenerator.generateContent("Foo", withNeutrons = true)
        assertTrue(content.contains("enum FooNeutron"), "Must generate FooNeutron enum")
        assertTrue(content.contains("neutrons = FooNeutron"), "Reactor macro must reference FooNeutron")
        assertTrue(content.contains("fn on_event"), "Must generate on_event handler")
        assertTrue(content.contains("match neutron"), "on_event must contain match expression")
    }

    @Test
    fun `name is substituted correctly throughout the file`() {
        val content = ReactorGenerator.generateContent("CartItem", withNeutrons = true)
        assertTrue(content.contains("CartItemState"), "State struct name substituted")
        assertTrue(content.contains("CartItemReactor"), "Reactor struct name substituted")
        assertTrue(content.contains("CartItemNeutron"), "Neutron enum name substituted")
        assertFalse(content.contains("\${name}"), "Template placeholders must not appear in output")
    }

    @Test
    fun `withoutNeutrons includes impl Default for state`() {
        val content = ReactorGenerator.generateContent("Theme", withNeutrons = false)
        assertTrue(content.contains("impl Default for ThemeState"), "Must include Default impl for state")
    }

    @Test
    fun `withNeutrons includes impl Default for state`() {
        val content = ReactorGenerator.generateContent("Theme", withNeutrons = true)
        assertTrue(content.contains("impl Default for ThemeState"), "Must include Default impl for state")
    }

    @Test
    fun `generated file starts with gloc prelude import`() {
        val withoutNeutrons = ReactorGenerator.generateContent("Counter", withNeutrons = false)
        val withNeutrons = ReactorGenerator.generateContent("Counter", withNeutrons = true)
        assertTrue(withoutNeutrons.startsWith("use gloc::prelude::*;"), "Must import gloc prelude")
        assertTrue(withNeutrons.startsWith("use gloc::prelude::*;"), "Must import gloc prelude")
    }
}
