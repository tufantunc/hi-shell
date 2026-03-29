import { SITE_URL } from '$lib/seo';

export const prerender = true;

const STATIC_PAGES = [
	{ path: '/', changefreq: 'weekly', priority: '1.0' },
	{ path: '/docs', changefreq: 'weekly', priority: '0.9' },
	{ path: '/docs/getting-started', changefreq: 'weekly', priority: '0.8' },
	{ path: '/docs/providers/embedded', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/providers/ollama', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/providers/lm-studio', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/providers/openrouter', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/providers/anthropic', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/providers/openai', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/configuration', changefreq: 'monthly', priority: '0.7' },
	{ path: '/docs/troubleshooting', changefreq: 'monthly', priority: '0.6' },
	{ path: '/docs/faq', changefreq: 'monthly', priority: '0.6' },
	{ path: '/changelog', changefreq: 'daily', priority: '0.8' },
	{ path: '/playground', changefreq: 'monthly', priority: '0.5' }
];

export async function GET() {
	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${STATIC_PAGES.map(
		(page) => `  <url>
    <loc>${SITE_URL}${page.path}</loc>
    <changefreq>${page.changefreq}</changefreq>
    <priority>${page.priority}</priority>
  </url>`
	).join('\n')}
</urlset>`;

	return new Response(xml.trim(), {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'max-age=3600'
		}
	});
}
