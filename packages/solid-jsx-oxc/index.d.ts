/**
 * solid-jsx-oxc - OXC-based JSX compiler for SolidJS
 */

export interface TransformOptions {
  /**
   * The module to import runtime helpers from
   * @default 'solid-js/web'
   */
  moduleName?: string;

  /**
   * Generate mode
   * @default 'dom'
   */
  generate?: 'dom' | 'ssr' | 'universal';

  /**
   * Enable hydration support
   * @default false
   */
  hydratable?: boolean;

  /**
   * Delegate events for better performance
   * @default true
   */
  delegateEvents?: boolean;

  /**
   * Additional events to delegate
   */
  delegatedEvents?: string[];

  /**
   * Wrap conditionals in memos
   * @default true
   */
  wrapConditionals?: boolean;

  /**
   * Pass context to custom elements
   * @default true
   */
  contextToCustomElements?: boolean;

  /**
   * Built-in components that should be passed through
   */
  builtIns?: string[];

  /**
   * Source filename for source maps
   */
  filename?: string;

  /**
   * Enable source maps
   * @default false
   */
  sourceMap?: boolean;
}

export interface TransformResult {
  /**
   * The transformed code
   */
  code: string;

  /**
   * Source map (if sourceMap was enabled)
   */
  map?: string;
}

export interface PresetResult {
  /**
   * The merged options
   */
  options: TransformOptions;

  /**
   * Transform function with the preset options
   */
  transform: (source: string) => TransformResult;
}

/**
 * Default transform options
 */
export const defaultOptions: TransformOptions;

/**
 * Transform JSX source code
 * @param source - The source code to transform
 * @param options - Transform options
 */
export function transform(source: string, options?: TransformOptions): TransformResult;

/**
 * Create a preset configuration (for compatibility with babel-preset-solid)
 * @param context - Babel context (ignored, for compatibility)
 * @param options - User options
 */
export function preset(context: unknown, options?: TransformOptions): PresetResult;

export default {
  transform,
  preset,
  defaultOptions,
};
