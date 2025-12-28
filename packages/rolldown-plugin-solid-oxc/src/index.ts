/**
 * Rolldown plugin for SolidJS using OXC-based compiler
 *
 * Since Rolldown uses OXC internally, this provides optimal performance.
 */

import type { Plugin } from 'rolldown';

export type FilterPattern = RegExp | string | (RegExp | string)[] | null | undefined;

/**
 * Simple filter function for include/exclude patterns
 */
function createFilter(
  include?: FilterPattern,
  exclude?: FilterPattern
): (id: string) => boolean {
  const toArray = (pattern: FilterPattern): (RegExp | string)[] => {
    if (pattern == null) return [];
    if (Array.isArray(pattern)) return pattern;
    return [pattern];
  };

  const includePatterns = toArray(include);
  const excludePatterns = toArray(exclude);

  const matches = (id: string, pattern: RegExp | string): boolean => {
    if (pattern instanceof RegExp) return pattern.test(id);
    return id.includes(pattern);
  };

  return (id: string) => {
    // If no include patterns, include everything by default
    const included = includePatterns.length === 0 || includePatterns.some(p => matches(id, p));
    const excluded = excludePatterns.some(p => matches(id, p));
    return included && !excluded;
  };
}

export interface SolidOxcOptions {
  /**
   * Dev mode - enables additional debugging
   * @default false
   */
  dev?: boolean;

  /**
   * Hot module replacement (requires dev: true)
   * @default true in dev mode
   */
  hot?: boolean;
  /**
   * Filter which files to transform
   * @default /\.[jt]sx$/
   */
  include?: FilterPattern;

  /**
   * Filter which files to exclude
   * @default /node_modules/
   */
  exclude?: FilterPattern;

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
   * Enable SSR mode
   * @default false
   */
  ssr?: boolean;
}

const defaultOptions: SolidOxcOptions = {
  include: /\.[jt]sx$/,
  exclude: /node_modules/,
  moduleName: 'solid-js/web',
  generate: 'dom',
  hydratable: false,
  delegateEvents: true,
  wrapConditionals: true,
  contextToCustomElements: true,
  dev: false,
  hot: true,
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
    'ErrorBoundary',
  ],
};

/**
 * Rolldown/Rollup plugin for SolidJS using OXC-based compiler
 */
export default function solidOxc(options: SolidOxcOptions = {}): Plugin {
  const opts = { ...defaultOptions, ...options };
  const filter = createFilter(opts.include, opts.exclude);

  // Lazy load the native module
  let solidJsxOxc: typeof import('solid-jsx-oxc') | null = null;

  return {
    name: 'rolldown-plugin-solid-oxc',

    async buildStart() {
      try {
        solidJsxOxc = await import('solid-jsx-oxc');
      } catch (e) {
        this.error(
          'Failed to load solid-jsx-oxc. Make sure it is built for your platform.'
        );
      }
    },

    async transform(code, id) {
      // Strip query parameters (e.g., ?v=123 from Vite/dev servers)
      const fileId = id.split('?', 1)[0];

      if (!filter(fileId)) {
        return null;
      }

      if (!solidJsxOxc) {
        this.error('solid-jsx-oxc module not loaded');
        return null;
      }

      const generate = opts.ssr ? 'ssr' : opts.generate;

      try {
        const result = solidJsxOxc.transformJsx(code, {
          filename: fileId,
          moduleName: opts.moduleName,
          generate,
          hydratable: opts.hydratable,
          delegateEvents: opts.delegateEvents,
          wrapConditionals: opts.wrapConditionals,
          contextToCustomElements: opts.contextToCustomElements,
          sourceMap: true,
        });

        let finalCode = result.code;

        // Add HMR support in dev mode
        if (opts.dev && opts.hot !== false) {
          const hotCode = `
if (import.meta.hot) {
  import.meta.hot.accept();
}
`;
          finalCode = finalCode + hotCode;
        }

        return {
          code: finalCode,
          map: result.map ? JSON.parse(result.map) : null,
        };
      } catch (e: unknown) {
        const message = e instanceof Error ? e.message : String(e);
        this.error(`Failed to transform ${id}: ${message}`);
        return null;
      }
    },

    // Ensure proper resolution of solid-js
    resolveId(source) {
      // Let Rolldown handle solid-js resolution
      return null;
    },
  };
}

// Named export for compatibility
export { solidOxc };
