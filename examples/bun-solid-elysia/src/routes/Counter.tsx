import { createSignal } from 'solid-js';

export function Counter() {
  const [count, setCount] = createSignal(0);

  return (
    <div>
      <h1>Counter</h1>
      <div class="card">
        <p>Interactive counter - works after hydration!</p>
        <p class="count">Count: {count()}</p>
        <div class="buttons">
          <button onClick={() => setCount(c => c - 1)}>-1</button>
          <button onClick={() => setCount(c => c + 1)}>+1</button>
          <button onClick={() => setCount(0)}>Reset</button>
        </div>
      </div>
    </div>
  );
}
