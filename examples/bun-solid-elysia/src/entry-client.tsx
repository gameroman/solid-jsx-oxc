import { hydrate } from 'solid-js/web';
import { App } from './App';

// Hydrate the server-rendered HTML
hydrate(() => <App />, document.getElementById('app')!);
