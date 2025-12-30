import { createSignal, onMount, Show } from 'solid-js';
import { isServer } from 'solid-js/web';

interface Stats {
  uptime: number;
  memory: number;
  requests: number;
}

export function Home() {
  const [stats, setStats] = createSignal<Stats | null>(null);
  const [loading, setLoading] = createSignal(false);

  // Only fetch on client after hydration
  onMount(async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/stats');
      setStats(await res.json());
    } catch (e) {
      console.error('Failed to fetch stats:', e);
    }
    setLoading(false);
  });

  return (
    <div>
      <h1>Bun + Solid + Elysia SSR</h1>
      <div class="card">
        <p>A full-stack Solid app with:</p>
        <ul>
          <li><strong>Bun</strong> - Runtime & bundler</li>
          <li><strong>solid-jsx-oxc</strong> - Native JSX compiler</li>
          <li><strong>Elysia</strong> - Server framework</li>
          <li><strong>SSR</strong> - Server-side rendering with hydration</li>
        </ul>
      </div>
      <div class="card">
        <h3>Server Stats (from API)</h3>
        <Show when={!isServer} fallback={<p>Loading on client...</p>}>
          <Show when={loading()}>
            <p>Fetching...</p>
          </Show>
          <Show when={stats()}>
            {(s) => (
              <ul>
                <li>Uptime: {s().uptime}s</li>
                <li>Memory: {s().memory}MB</li>
                <li>Request count: {s().requests}</li>
              </ul>
            )}
          </Show>
        </Show>
      </div>
    </div>
  );
}
