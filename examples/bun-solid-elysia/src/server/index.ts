import { Elysia } from 'elysia';
import { renderToStringAsync } from 'solid-js/web';
import { App } from '../App';

const isDev = process.env.NODE_ENV !== 'production';
const port = process.env.PORT || 3000;

// HTML template for SSR
function htmlTemplate(appHtml: string, clientScript: string) {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Bun + Solid + Elysia</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: system-ui, sans-serif; line-height: 1.6; }
    .container { max-width: 800px; margin: 0 auto; padding: 2rem; }
    nav { display: flex; gap: 1rem; margin-bottom: 2rem; }
    nav a { color: #0066cc; text-decoration: none; }
    nav a:hover { text-decoration: underline; }
    h1 { margin-bottom: 1rem; }
    .card { background: #f5f5f5; padding: 1rem; border-radius: 8px; margin: 1rem 0; }
    button { padding: 0.5rem 1rem; cursor: pointer; }
  </style>
</head>
<body>
  <div id="app">${appHtml}</div>
  <script type="module">${clientScript}</script>
</body>
</html>`;
}

// Create Elysia server
const app = new Elysia()
  // Serve static files in production
  .get('/dist/*', async ({ params }) => {
    const file = Bun.file(`./dist/${params['*']}`);
    if (await file.exists()) {
      return new Response(file, {
        headers: { 'Content-Type': 'application/javascript' },
      });
    }
    return new Response('Not found', { status: 404 });
  })

  // SSR handler for all routes
  .get('*', async ({ request }) => {
    const url = new URL(request.url);

    try {
      // Render the app to string
      const appHtml = await renderToStringAsync(() => App({ url: url.pathname }));

      // Client-side hydration script
      const clientScript = isDev
        ? `import('/dist/client.js');`
        : `import('/dist/client.js');`;

      return new Response(htmlTemplate(appHtml, clientScript), {
        headers: { 'Content-Type': 'text/html' },
      });
    } catch (error) {
      console.error('SSR Error:', error);
      return new Response('Server Error', { status: 500 });
    }
  })

  .listen(port);

console.log(`Server running at http://localhost:${port}`);
console.log(`Mode: ${isDev ? 'development' : 'production'}`);

export type App = typeof app;
