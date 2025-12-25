/**
 * solid-jsx-oxc - OXC-based JSX compiler for SolidJS (ESM)
 */

import { createRequire } from 'module';
const require = createRequire(import.meta.url);

// Try to load the native module
let nativeBinding = null;

try {
  nativeBinding = require('./solid-jsx-oxc.node');
} catch (e) {
  console.warn('solid-jsx-oxc: Native module not found. Run `npm run build` to compile.');
}

/**
 * Default options matching babel-preset-solid
 */
export const defaultOptions = {
  moduleName: 'solid-js/web',
  builtIns: [
    'For',
    'Show',
    'Switch',
    'Match',
    'Suspense',
    'SuspenseList',
    'Portal',
    'Index',
    'Dynamic',
    'ErrorBoundary'
  ],
  contextToCustomElements: true,
  wrapConditionals: true,
  generate: 'dom',
  hydratable: false,
  delegateEvents: true,
  sourceMap: false,
};

/**
 * Transform JSX source code
 */
export function transform(source, options = {}) {
  if (!nativeBinding) {
    throw new Error('solid-jsx-oxc: Native module not loaded.');
  }

  const mergedOptions = { ...defaultOptions, ...options };
  return nativeBinding.transformJsx(source, mergedOptions);
}

/**
 * Create a preset configuration
 */
export function preset(context, options = {}) {
  const mergedOptions = { ...defaultOptions, ...options };

  return {
    options: mergedOptions,
    transform: (source) => transform(source, mergedOptions),
  };
}

export default {
  transform,
  preset,
  defaultOptions,
};
