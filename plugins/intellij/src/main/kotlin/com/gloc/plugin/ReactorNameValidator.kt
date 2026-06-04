package com.gloc.plugin

internal val PASCAL_CASE_PATTERN = Regex("^[A-Z][a-zA-Z0-9]*$")

/** Returns null when [name] is valid, or a human-readable error message when it is not. */
fun validateReactorName(name: String): String? = when {
    name.isEmpty() -> "Reactor name is required."
    !PASCAL_CASE_PATTERN.matches(name) ->
        "Name must be PascalCase and start with an uppercase letter (e.g. Counter, CartItem)."
    else -> null
}
