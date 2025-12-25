//! SSR component transform
//!
//! Components in SSR are rendered the same way as DOM - using createComponent.
//! The component itself decides whether to render for server or client.

use oxc_ast::ast::{
    JSXElement, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue,
};

use common::{TransformOptions, is_built_in, expr_to_string};

use crate::ir::{SSRContext, SSRResult};

/// Helper to find a prop value by name
fn find_prop_expr<'a>(element: &'a JSXElement<'a>, name: &str) -> Option<String> {
    for attr in &element.opening_element.attributes {
        if let JSXAttributeItem::Attribute(attr) = attr {
            let key = match &attr.name {
                JSXAttributeName::Identifier(id) => id.name.as_str(),
                _ => continue,
            };

            if key == name {
                return match &attr.value {
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        container.expression.as_expression()
                            .map(|e| expr_to_string(e))
                    }
                    Some(JSXAttributeValue::StringLiteral(lit)) => {
                        Some(format!("\"{}\"", lit.value))
                    }
                    None => Some("true".to_string()),
                    _ => None,
                };
            }
        }
    }
    None
}

/// Get children callback expression
fn get_children_expr<'a>(element: &'a JSXElement<'a>) -> String {
    if element.children.is_empty() {
        return "undefined".to_string();
    }

    // For now, we just serialize the children as a function
    // The actual implementation would recursively transform children
    "() => /* children */".to_string()
}

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
    element: &JSXElement<'a>,
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
            let each = find_prop_expr(element, "each").unwrap_or("[]".to_string());
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(For, {{ each: {}, children: {} }})", each, children),
                false,
                false,
            );
        }

        "Show" => {
            context.register_helper("Show");
            let when = find_prop_expr(element, "when").unwrap_or("false".to_string());
            let fallback = find_prop_expr(element, "fallback").unwrap_or("undefined".to_string());
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(Show, {{ when: {}, fallback: {}, children: {} }})", when, fallback, children),
                false,
                false,
            );
        }

        "Switch" => {
            context.register_helper("Switch");
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(Switch, {{ children: {} }})", children),
                false,
                false,
            );
        }

        "Match" => {
            context.register_helper("Match");
            let when = find_prop_expr(element, "when").unwrap_or("false".to_string());
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(Match, {{ when: {}, children: {} }})", when, children),
                false,
                false,
            );
        }

        "Index" => {
            context.register_helper("Index");
            let each = find_prop_expr(element, "each").unwrap_or("[]".to_string());
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(Index, {{ each: {}, children: {} }})", each, children),
                false,
                false,
            );
        }

        "Suspense" => {
            context.register_helper("Suspense");
            let fallback = find_prop_expr(element, "fallback").unwrap_or("undefined".to_string());
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(Suspense, {{ fallback: {}, children: {} }})", fallback, children),
                false,
                false,
            );
        }

        "Portal" => {
            // Portal in SSR just renders children (no mount target on server)
            context.register_helper("Portal");
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(Portal, {{ children: {} }})", children),
                false,
                false,
            );
        }

        "Dynamic" => {
            context.register_helper("Dynamic");
            let component = find_prop_expr(element, "component").unwrap_or("undefined".to_string());
            result.push_dynamic(
                format!("createComponent(Dynamic, {{ component: {} }})", component),
                false,
                false,
            );
        }

        "ErrorBoundary" => {
            context.register_helper("ErrorBoundary");
            let fallback = find_prop_expr(element, "fallback").unwrap_or("undefined".to_string());
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(ErrorBoundary, {{ fallback: {}, children: {} }})", fallback, children),
                false,
                false,
            );
        }

        "NoHydration" => {
            // Special SSR component - renders children without hydration markers
            context.register_helper("NoHydration");
            let children = get_children_expr(element);
            result.push_dynamic(
                format!("createComponent(NoHydration, {{ children: {} }})", children),
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
                        if let Some(expr) = container.expression.as_expression() {
                            let expr_str = expr_to_string(expr);
                            // For SSR, we still need getters for lazy evaluation
                            dynamic_props.push(format!(
                                "get {}() {{ return {}; }}",
                                key, expr_str
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
