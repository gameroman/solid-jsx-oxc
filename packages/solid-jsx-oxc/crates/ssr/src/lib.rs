//! SSR (Server-Side Rendering) transform for SolidJS
//!
//! This crate generates SSR output that uses template strings and
//! escape() calls instead of DOM operations.
//!
//! ## Output Format
//!
//! ```js
//! // Input JSX
//! <div class={style()}>{count()}</div>
//!
//! // SSR Output
//! ssr`<div${ssrHydrationKey()} class="${escape(style(), true)}">${escape(count())}</div>`
//! ```

pub mod transform;
pub mod element;
pub mod component;
pub mod template;
pub mod ir;

pub use transform::*;
