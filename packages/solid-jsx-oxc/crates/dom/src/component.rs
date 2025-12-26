//! Component transform
//! Handles <MyComponent /> -> createComponent(MyComponent, {...})

use oxc_ast::ast::{
    JSXElement, JSXAttribute, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue, JSXChild,
};

use common::{TransformOptions, is_built_in, is_dynamic, expr_to_string};

use crate::ir::{BlockContext, TransformResult, Expr};

/// Transform a component element
pub fn transform_component<'a>(
    element: &JSXElement<'a>,
    tag_name: &str,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) -> TransformResult {
    let mut result = TransformResult::default();

    // Check if this is a built-in (For, Show, etc.)
    if is_built_in(tag_name) {
        return transform_builtin(element, tag_name, context, options);
    }

    context.register_helper("createComponent");

    // Build props object
    let props = build_props(element, context, options);

    // Generate createComponent call
    result.exprs.push(Expr {
        code: format!("createComponent({}, {})", tag_name, props),
    });

    result
}

/// Transform built-in control flow components
fn transform_builtin<'a>(
    element: &JSXElement<'a>,
    tag_name: &str,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) -> TransformResult {
    let mut result = TransformResult::default();

    match tag_name {
        "For" => transform_for(element, &mut result, context, options),
        "Show" => transform_show(element, &mut result, context, options),
        "Switch" => transform_switch(element, &mut result, context, options),
        "Match" => transform_match(element, &mut result, context, options),
        "Index" => transform_index(element, &mut result, context, options),
        "Suspense" => transform_suspense(element, &mut result, context, options),
        "Portal" => transform_portal(element, &mut result, context, options),
        "Dynamic" => transform_dynamic(element, &mut result, context, options),
        "ErrorBoundary" => transform_error_boundary(element, &mut result, context, options),
        _ => {
            // Fallback to regular component transform
            context.register_helper("createComponent");
            result.exprs.push(Expr {
                code: format!("createComponent({}, {{}})", tag_name),
            });
        }
    }

    result
}

/// Transform <For each={...}>{item => ...}</For>
fn transform_for<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    _options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("For");

    // Get the 'each' prop
    let each_expr = find_prop(element, "each")
        .and_then(|attr| attr.value.as_ref())
        .and_then(|v| match v {
            JSXAttributeValue::ExpressionContainer(c) => c.expression.as_expression(),
            _ => None,
        })
        .map(|e| expr_to_string(e))
        .unwrap_or_else(|| "undefined".to_string());

    // Get the children (callback function)
    let children = get_children_callback(element);

    result.exprs.push(Expr {
        code: format!(
            "createComponent(For, {{ each: {}, children: {} }})",
            each_expr, children
        ),
    });
}

/// Transform <Show when={...} fallback={...}>...</Show>
fn transform_show<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    _options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Show");

    // Get the 'when' prop
    let when_expr = find_prop(element, "when")
        .and_then(|attr| attr.value.as_ref())
        .and_then(|v| match v {
            JSXAttributeValue::ExpressionContainer(c) => c.expression.as_expression(),
            _ => None,
        })
        .map(|e| expr_to_string(e))
        .unwrap_or_else(|| "undefined".to_string());

    // Get the 'fallback' prop
    let fallback_expr = find_prop(element, "fallback")
        .and_then(|attr| attr.value.as_ref())
        .and_then(|v| match v {
            JSXAttributeValue::ExpressionContainer(c) => c.expression.as_expression(),
            _ => None,
        })
        .map(|e| expr_to_string(e))
        .unwrap_or_else(|| "undefined".to_string());

    // Get the children
    let children = get_children_callback(element);

    result.exprs.push(Expr {
        code: format!(
            "createComponent(Show, {{ when: {}, fallback: {}, children: {} }})",
            when_expr, fallback_expr, children
        ),
    });
}

/// Transform <Switch>...</Switch>
fn transform_switch<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Switch");

    result.exprs.push(Expr {
        code: format!("_createComponent(_Switch, {{ children: /* Match children */ }})"),
    });
}

/// Transform <Match when={...}>...</Match>
fn transform_match<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Match");

    result.exprs.push(Expr {
        code: format!("_createComponent(_Match, {{ when: /* when */, children: /* children */ }})"),
    });
}

/// Transform <Index each={...}>{(item, index) => ...}</Index>
fn transform_index<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Index");

    result.exprs.push(Expr {
        code: format!(
            "_createComponent(_Index, {{ each: /* each */, children: /* callback */ }})"
        ),
    });
}

/// Transform <Suspense fallback={...}>...</Suspense>
fn transform_suspense<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Suspense");

    result.exprs.push(Expr {
        code: format!(
            "_createComponent(_Suspense, {{ fallback: /* fallback */, children: /* children */ }})"
        ),
    });
}

/// Transform <Portal mount={...}>...</Portal>
fn transform_portal<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Portal");

    result.exprs.push(Expr {
        code: format!(
            "_createComponent(_Portal, {{ mount: /* mount */, children: /* children */ }})"
        ),
    });
}

/// Transform <Dynamic component={...} {...props} />
fn transform_dynamic<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("Dynamic");

    result.exprs.push(Expr {
        code: format!(
            "_createComponent(_Dynamic, {{ component: /* component */, .../* props */ }})"
        ),
    });
}

/// Transform <ErrorBoundary fallback={...}>...</ErrorBoundary>
fn transform_error_boundary<'a>(
    element: &JSXElement<'a>,
    result: &mut TransformResult,
    context: &BlockContext,
    options: &TransformOptions<'a>,
) {
    context.register_helper("createComponent");
    context.register_helper("ErrorBoundary");

    result.exprs.push(Expr {
        code: format!(
            "_createComponent(_ErrorBoundary, {{ fallback: /* fallback */, children: /* children */ }})"
        ),
    });
}

/// Build props object for a component
fn build_props<'a>(
    element: &JSXElement<'a>,
    context: &BlockContext,
    _options: &TransformOptions<'a>,
) -> String {
    let mut static_props: Vec<String> = vec![];
    let mut dynamic_props: Vec<String> = vec![];
    let mut spreads: Vec<String> = vec![];

    for attr in &element.opening_element.attributes {
        match attr {
            JSXAttributeItem::Attribute(attr) => {
                let key = match &attr.name {
                    JSXAttributeName::Identifier(id) => id.name.to_string(),
                    JSXAttributeName::NamespacedName(ns) => {
                        format!("{}:{}", ns.namespace.name, ns.name.name)
                    }
                };

                match &attr.value {
                    Some(JSXAttributeValue::StringLiteral(lit)) => {
                        static_props.push(format!("{}: \"{}\"", key, lit.value));
                    }
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        if let Some(expr) = container.expression.as_expression() {
                            let expr_str = expr_to_string(expr);
                            if is_dynamic(expr) {
                                // Dynamic prop - use getter
                                dynamic_props.push(format!(
                                    "get {}() {{ return {}; }}",
                                    key, expr_str
                                ));
                            } else {
                                static_props.push(format!("{}: {}", key, expr_str));
                            }
                        }
                    }
                    None => {
                        static_props.push(format!("{}: true", key));
                    }
                    _ => {}
                }
            }
            JSXAttributeItem::SpreadAttribute(spread) => {
                spreads.push(expr_to_string(&spread.argument));
            }
        }
    }

    // Handle children
    if !element.children.is_empty() {
        let children_expr = get_children_expr(element, context);
        if !children_expr.is_empty() {
            dynamic_props.push(format!("get children() {{ return {}; }}", children_expr));
        }
    }

    // Combine all props
    let all_props = static_props.into_iter()
        .chain(dynamic_props)
        .collect::<Vec<_>>()
        .join(", ");

    // Combine props
    if !spreads.is_empty() {
        context.register_helper("mergeProps");
        let spread_list = spreads.join(", ");
        if all_props.is_empty() {
            format!("mergeProps({})", spread_list)
        } else {
            format!("mergeProps({}, {{ {} }})", spread_list, all_props)
        }
    } else if all_props.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", all_props)
    }
}

/// Get children as an expression
fn get_children_expr<'a>(
    element: &JSXElement<'a>,
    _context: &BlockContext,
) -> String {
    let mut children: Vec<String> = vec![];

    for child in &element.children {
        match child {
            JSXChild::Text(text) => {
                let content = common::expression::trim_whitespace(&text.value);
                if !content.is_empty() {
                    children.push(format!("\"{}\"", common::expression::escape_html(&content, false)));
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if let Some(expr) = container.expression.as_expression() {
                    children.push(expr_to_string(expr));
                }
            }
            JSXChild::Element(child_elem) => {
                // Nested JSX element - this will be transformed by the traversal
                // For now, output a placeholder that represents the element call
                let tag = common::get_tag_name(child_elem);
                if common::is_component(&tag) {
                    // Component
                    children.push(format!("/* <{}> */", tag));
                } else {
                    // Native element - would be a template clone
                    children.push(format!("/* <{}> */", tag));
                }
            }
            JSXChild::Fragment(_) => {
                children.push("/* fragment */".to_string());
            }
            JSXChild::Spread(spread) => {
                children.push(expr_to_string(&spread.expression));
            }
        }
    }

    if children.len() == 1 {
        children.pop().unwrap_or_default()
    } else if children.is_empty() {
        String::new()
    } else {
        format!("[{}]", children.join(", "))
    }
}

/// Find a prop by name
fn find_prop<'a>(element: &'a JSXElement<'a>, name: &str) -> Option<&'a JSXAttribute<'a>> {
    for attr in &element.opening_element.attributes {
        if let JSXAttributeItem::Attribute(attr) = attr {
            if let JSXAttributeName::Identifier(id) = &attr.name {
                if id.name == name {
                    return Some(attr);
                }
            }
        }
    }
    None
}

/// Get children as a callback function (for For, Index, etc.)
fn get_children_callback<'a>(element: &JSXElement<'a>) -> String {
    // The children of For/Index should be an arrow function
    // <For each={items}>{item => <div>{item}</div>}</For>
    for child in &element.children {
        if let JSXChild::ExpressionContainer(container) = child {
            if let Some(expr) = container.expression.as_expression() {
                // This should be an arrow function like: item => <div>{item}</div>
                return expr_to_string(expr);
            }
        }
    }
    // If no expression child found, return a no-op function
    "() => undefined".to_string()
}
