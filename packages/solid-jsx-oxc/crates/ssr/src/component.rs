//! SSR component transform
//!
//! Components in SSR are rendered the same way as DOM - using createComponent.
//! The component itself decides whether to render for server or client.

use oxc_ast::ast::{
    JSXElement, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue,
};

use common::{TransformOptions, is_built_in};

use crate::ir::{SSRContext, SSRResult};

/// Transform a component for SSR
pub fn transform_component<'a>(
    element: &JSXElement<'a>,
    tag_name: &str,
    context: &SSRContext,
    options: &TransformOptions<'a>,
) -> SSRResult {
    let mut result = SSRResult::new();

    // Check if this is a built-in (For, Show, etc.)
    if is_built_in(tag_name) {
        return transform_builtin(element, tag_name, context, options);
    }

    context.register_helper("createComponent");
    context.register_helper("escape");

    // Build props
    let props = build_props(element, context, options);

    // Generate createComponent call - will be escaped by parent
    result.push_dynamic(
        format!("createComponent({}, {})", tag_name, props),
        false,
        false, // Components return escaped content
    );

    result
}

/// Transform built-in control flow components for SSR
fn transform_builtin<'a>(
    _element: &JSXElement<'a>,
    tag_name: &str,
    context: &SSRContext,
    _options: &TransformOptions<'a>,
) -> SSRResult {
    let mut result = SSRResult::new();

    context.register_helper("createComponent");
    context.register_helper("escape");

    match tag_name {
        "For" => {
            context.register_helper("For");
            result.push_dynamic(
                "createComponent(For, { each: /* each */, children: /* callback */ })".to_string(),
                false,
                false,
            );
        }

        "Show" => {
            context.register_helper("Show");
            result.push_dynamic(
                "createComponent(Show, { when: /* when */, fallback: /* fallback */, children: /* children */ })".to_string(),
                false,
                false,
            );
        }

        "Switch" => {
            context.register_helper("Switch");
            result.push_dynamic(
                "createComponent(Switch, { children: /* Match children */ })".to_string(),
                false,
                false,
            );
        }

        "Match" => {
            context.register_helper("Match");
            result.push_dynamic(
                "createComponent(Match, { when: /* when */, children: /* children */ })".to_string(),
                false,
                false,
            );
        }

        "Index" => {
            context.register_helper("Index");
            result.push_dynamic(
                "createComponent(Index, { each: /* each */, children: /* callback */ })".to_string(),
                false,
                false,
            );
        }

        "Suspense" => {
            context.register_helper("Suspense");
            result.push_dynamic(
                "createComponent(Suspense, { fallback: /* fallback */, children: /* children */ })".to_string(),
                false,
                false,
            );
        }

        "Portal" => {
            // Portal in SSR just renders children (no mount target on server)
            context.register_helper("Portal");
            result.push_dynamic(
                "createComponent(Portal, { children: /* children */ })".to_string(),
                false,
                false,
            );
        }

        "Dynamic" => {
            context.register_helper("Dynamic");
            result.push_dynamic(
                "createComponent(Dynamic, { component: /* component */, .../* props */ })".to_string(),
                false,
                false,
            );
        }

        "ErrorBoundary" => {
            context.register_helper("ErrorBoundary");
            result.push_dynamic(
                "createComponent(ErrorBoundary, { fallback: /* fallback */, children: /* children */ })".to_string(),
                false,
                false,
            );
        }

        "NoHydration" => {
            // Special SSR component - renders children without hydration markers
            context.register_helper("NoHydration");
            result.push_dynamic(
                "createComponent(NoHydration, { children: /* children */ })".to_string(),
                false,
                true, // Don't escape - it handles its own output
            );
        }

        _ => {
            // Unknown built-in, treat as regular component
            result.push_dynamic(
                format!("createComponent({}, {{}})", tag_name),
                false,
                false,
            );
        }
    }

    result
}

/// Build props object for a component
fn build_props<'a>(
    element: &JSXElement<'a>,
    context: &SSRContext,
    _options: &TransformOptions<'a>,
) -> String {
    let mut static_props: Vec<String> = vec![];
    let mut dynamic_props: Vec<String> = vec![];
    let mut has_spread = false;

    for attr in &element.opening_element.attributes {
        match attr {
            JSXAttributeItem::Attribute(attr) => {
                let key = match &attr.name {
                    JSXAttributeName::Identifier(id) => id.name.to_string(),
                    JSXAttributeName::NamespacedName(ns) => {
                        format!("{}:{}", ns.namespace.name, ns.name.name)
                    }
                };

                // Skip event handlers in SSR
                if key.starts_with("on") || key == "ref" || key.starts_with("use:") {
                    continue;
                }

                match &attr.value {
                    Some(JSXAttributeValue::StringLiteral(lit)) => {
                        static_props.push(format!("{}: \"{}\"", key, lit.value));
                    }
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        if let Some(_expr) = container.expression.as_expression() {
                            // For SSR, we still need getters for lazy evaluation
                            dynamic_props.push(format!(
                                "get {}() {{ return /* expr */; }}",
                                key
                            ));
                        }
                    }
                    None => {
                        static_props.push(format!("{}: true", key));
                    }
                    _ => {}
                }
            }
            JSXAttributeItem::SpreadAttribute(_) => {
                has_spread = true;
            }
        }
    }

    // Handle children
    if !element.children.is_empty() {
        dynamic_props.push("get children() { return /* children */; }".to_string());
    }

    // Combine props
    if has_spread {
        context.register_helper("mergeProps");
        format!(
            "mergeProps(/* spread */, {{ {} }})",
            static_props.into_iter().chain(dynamic_props).collect::<Vec<_>>().join(", ")
        )
    } else if dynamic_props.is_empty() && static_props.is_empty() {
        "{}".to_string()
    } else {
        format!(
            "{{ {} }}",
            static_props.into_iter().chain(dynamic_props).collect::<Vec<_>>().join(", ")
        )
    }
}
