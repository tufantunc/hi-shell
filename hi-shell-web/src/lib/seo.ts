export const SITE_URL = 'https://hi-shell.dev';
export const SITE_NAME = 'hi-shell';
export const SITE_DESCRIPTION =
	'An intelligent terminal assistant that translates natural language descriptions into executable shell commands.';

export interface SeoData {
	title: string;
	description: string;
	path: string;
	ogImage?: string;
	noIndex?: boolean;
}

export function buildSeoMeta(data: SeoData) {
	const url = `${SITE_URL}${data.path}`;
	const ogImage = data.ogImage || `${SITE_URL}/small-hermit-crab-mascot.png`;

	return {
		title: `${data.title} | ${SITE_NAME}`,
		description: data.description,
		url,
		ogImage,
		noIndex: data.noIndex || false
	};
}

export function buildJsonLd() {
	return JSON.stringify({
		'@context': 'https://schema.org',
		'@type': 'SoftwareApplication',
		name: SITE_NAME,
		description: SITE_DESCRIPTION,
		url: SITE_URL,
		applicationCategory: 'DeveloperApplication',
		operatingSystem: 'macOS, Linux, Windows',
		offers: {
			'@type': 'Offer',
			price: '0',
			priceCurrency: 'USD'
		},
		license: 'https://github.com/tufantunc/hi-shell/blob/main/LICENSE',
		codeRepository: 'https://github.com/tufantunc/hi-shell',
		programmingLanguage: 'Rust',
		featureList: [
			'Embedded LLM models with Metal/CUDA acceleration',
			'Local LLM support via Ollama and LM Studio',
			'Cloud LLM integration (OpenRouter, Anthropic, OpenAI, Gemini)',
			'Interactive REPL mode',
			'Dangerous command detection',
			'Cross-platform support'
		]
	});
}
