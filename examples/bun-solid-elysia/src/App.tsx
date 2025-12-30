import { createSignal, Show } from 'solid-js';

// Simple app without router for initial SSR testing
export function App(props: { url?: string }) {
  return (
    <div class="container">
      <h1>Bun + Solid + Elysia SSR</h1>
      <div class="card">
        <p>Server rendered at: {props.url || '/'}</p>
        <p>This is a full-stack Solid app using:</p>
        <ul>
          <li><strong>Bun</strong> - Runtime & bundler (via bunfig.toml)</li>
          <li><strong>solid-jsx-oxc</strong> - Native JSX compiler</li>
          <li><strong>Elysia</strong> - Server framework</li>
        </ul>
      </div>
      <Counter />
    </div>
  );
}

function Counter() {
  const [count, setCount] = createSignal(0);

  return (
    <div class="card">
      <p>Interactive counter (works after hydration):</p>
      <p style={{ "font-size": "2rem", "margin": "1rem 0" }}>
        Count: {count()}
      </p>
      <div style={{ display: "flex", gap: "0.5rem" }}>
        <button onClick={() => setCount(c => c - 1)}>-1</button>
        <button onClick={() => setCount(c => c + 1)}>+1</button>
        <button onClick={() => setCount(0)}>Reset</button>
      </div>
    </div>
  );
}
