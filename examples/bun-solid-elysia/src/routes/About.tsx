export function About() {
  return (
    <div>
      <h1>About</h1>
      <div class="card">
        <h3>How SSR Works</h3>
        <ol>
          <li>Server renders HTML using <code>renderToStringAsync</code></li>
          <li>Browser receives pre-rendered HTML (fast first paint)</li>
          <li>Client bundle loads and hydrates for interactivity</li>
        </ol>
      </div>
      <div class="card">
        <h3>Build Process</h3>
        <ol>
          <li><code>bun-plugin-solid-oxc</code> transforms JSX</li>
          <li>Client bundle: DOM mode with hydration</li>
          <li>Server bundle: SSR mode with hydration markers</li>
        </ol>
      </div>
    </div>
  );
}
