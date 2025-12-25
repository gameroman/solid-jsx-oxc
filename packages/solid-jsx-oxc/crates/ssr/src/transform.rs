//! Main SSR transform logic
//!
//! This implements the Traverse trait to walk the AST and transform JSX for SSR.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, JSXElement, JSXFragment, JSXChild, JSXExpressionContainer,
    JSXText, Program,
};
use oxc_traverse::{Traverse, TraverseCtx, traverse_mut};
use oxc_semantic::SemanticBuilder;

use common::{TransformOptions, is_component, get_tag_name};

use crate::ir::{SSRContext, SSRResult};
use crate::element::transform_element;
use crate::component::transform_component;

/// The main SSR JSX transformer
pub struct SSRTransform<'a> {
    allocator: &'a Allocator,
    options: &'a TransformOptions<'a>,
    context: SSRContext,
}

impl<'a> SSRTransform<'a> {
    pub fn new(allocator: &'a Allocator, options: &'a TransformOptions<'a>) -> Self {
        Self {
            allocator,
            options,
            context: SSRContext::new(options.hydratable),
        }
    }

    /// Run the transform on a program
    pub fn transform(mut self, program: &mut Program<'a>) {
        let allocator = self.allocator as *const Allocator;
        traverse_mut(
            &mut self,
            unsafe { &*allocator },
            program,
            SemanticBuilder::new()
                .build(program)
                .semantic
                .into_scoping(),
            (),
        );
    }

    /// Transform a JSX node and return the SSR result
    fn transform_node(
        &self,
        node: &JSXChild<'a>,
    ) -> Option<SSRResult> {
        match node {
            JSXChild::Element(element) => {
                Some(self.transform_jsx_element(element))
            }
            JSXChild::Fragment(fragment) => {
                Some(self.transform_fragment(fragment))
            }
            JSXChild::Text(text) => {
                self.transform_text(text)
            }
            JSXChild::ExpressionContainer(container) => {
                self.transform_expression_container(container)
            }
            JSXChild::Spread(_) => {
                // Spread children - treat as dynamic
                let mut result = SSRResult::new();
                self.context.register_helper("escape");
                result.push_dynamic("/* spread */".to_string(), false, false);
                Some(result)
            }
        }
    }

    /// Transform a JSX element
    fn transform_jsx_element(
        &self,
        element: &JSXElement<'a>,
    ) -> SSRResult {
        let tag_name = get_tag_name(element);

        if is_component(&tag_name) {
            transform_component(element, &tag_name, &self.context, self.options)
        } else {
            transform_element(element, &tag_name, &self.context, self.options)
        }
    }

    /// Transform a JSX fragment
    fn transform_fragment(
        &self,
        fragment: &JSXFragment<'a>,
    ) -> SSRResult {
        let mut result = SSRResult::new();

        for child in &fragment.children {
            if let Some(child_result) = self.transform_node(child) {
                result.merge(child_result);
            }
        }

        result
    }

    /// Transform JSX text
    fn transform_text(&self, text: &JSXText<'a>) -> Option<SSRResult> {
        let content = common::expression::trim_whitespace(&text.value);
        if content.is_empty() {
            return None;
        }

        let mut result = SSRResult::new();
        result.push_static(&common::expression::escape_html(&content, false));
        Some(result)
    }

    /// Transform a JSX expression container
    fn transform_expression_container(
        &self,
        container: &JSXExpressionContainer<'a>,
    ) -> Option<SSRResult> {
        if let Some(_expr) = container.expression.as_expression() {
            self.context.register_helper("escape");
            let mut result = SSRResult::new();
            result.push_dynamic("/* expr */".to_string(), false, false);
            Some(result)
        } else {
            None
        }
    }
}

impl<'a> Traverse<'a, ()> for SSRTransform<'a> {
    fn enter_expression(
        &mut self,
        node: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        match node {
            Expression::JSXElement(element) => {
                let _result = self.transform_jsx_element(element);
                // TODO: Replace node with generated ssr call
            }
            Expression::JSXFragment(fragment) => {
                let _result = self.transform_fragment(fragment);
                // TODO: Replace node with generated ssr call
            }
            _ => {}
        }
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Generate imports for helpers at the top of the file
        let _helpers = self.context.helpers.borrow();
        // TODO: Insert import statements
    }
}
