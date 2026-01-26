import { env } from '$env/dynamic/private';

const POSTHOG_API_KEY = env.POSTHOG_API_KEY || '';
const POSTHOG_PROJECT_ID = env.POSTHOG_PROJECT_ID || '';

interface PostHogInsightResult {
	result?: Array<{ count?: number; data?: number[] }>;
}

interface ProviderStats {
	provider: string;
	model: string | null;
	count: number;
	avgLatencyMs: number;
}

interface Stats {
	totalUsers: number;
	totalCommands: number;
	githubStars: number;
	topProviders: ProviderStats[];
	lastUpdated: string;
}

async function queryPostHog(query: string): Promise<PostHogInsightResult | null> {
	if (!POSTHOG_API_KEY || !POSTHOG_PROJECT_ID) {
		return null;
	}

	try {
		const response = await fetch(
			`https://app.posthog.com/api/projects/${POSTHOG_PROJECT_ID}/query/`,
			{
				method: 'POST',
				headers: {
					Authorization: `Bearer ${POSTHOG_API_KEY}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					query: {
						kind: 'HogQLQuery',
						query
					}
				})
			}
		);

		if (!response.ok) {
			console.error('PostHog API error:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('PostHog fetch error:', error);
		return null;
	}
}

async function fetchGitHubStars(): Promise<number> {
	try {
		const response = await fetch('https://api.github.com/repos/tufantunc/hi-shell', {
			headers: { Accept: 'application/vnd.github.v3+json' }
		});
		if (!response.ok) return 0;
		const data = await response.json();
		return data.stargazers_count || 0;
	} catch {
		return 0;
	}
}

export async function fetchStats(): Promise<Stats> {
	const defaultStats: Stats = {
		totalUsers: 0,
		totalCommands: 0,
		githubStars: 0,
		topProviders: [],
		lastUpdated: new Date().toISOString()
	};

	try {
		const [githubStars, posthogStats] = await Promise.all([
			fetchGitHubStars(),
			fetchPostHogStats(defaultStats)
		]);

		return {
			...posthogStats,
			githubStars
		};
	} catch (error) {
		console.error('Failed to fetch stats:', error);
		return defaultStats;
	}
}

async function fetchPostHogStats(defaultStats: Stats): Promise<Stats> {
	if (!POSTHOG_API_KEY || !POSTHOG_PROJECT_ID) {
		console.log('PostHog credentials not configured, using default stats');
		return defaultStats;
	}

	try {
		const [usersResult, commandsResult, providersResult] = await Promise.all([
			queryPostHog(`SELECT count(DISTINCT distinct_id) as count FROM events WHERE timestamp > now() - INTERVAL 365 DAY`),
			queryPostHog(`SELECT count(*) as count FROM events WHERE event = 'command_generated' AND timestamp > now() - INTERVAL 365 DAY`),
			queryPostHog(`SELECT 
				JSONExtractString(properties, 'provider') as provider,
				JSONExtractString(properties, 'model') as model,
				count(*) as count,
				avg(JSONExtractFloat(properties, 'latency_ms')) as avg_latency
				FROM events 
				WHERE event = 'command_generated' AND timestamp > now() - INTERVAL 365 DAY
				GROUP BY provider, model
				ORDER BY count DESC
				LIMIT 5`)
		]);

		const topProviders = extractProviderStats(providersResult);

		const stats: Stats = {
			totalUsers: extractCount(usersResult) || defaultStats.totalUsers,
			totalCommands: extractCount(commandsResult) || defaultStats.totalCommands,
			githubStars: 0,
			topProviders,
			lastUpdated: new Date().toISOString()
		};

		return stats;
	} catch (error) {
		console.error('Failed to fetch PostHog stats:', error);
		return defaultStats;
	}
}

function extractCount(result: PostHogInsightResult | null): number {
	if (!result) return 0;
	try {
		const data = result as { results?: number[][] };
		if (data.results && data.results[0] && typeof data.results[0][0] === 'number') {
			return data.results[0][0];
		}
	} catch {
		return 0;
	}
	return 0;
}

function extractFloat(result: PostHogInsightResult | null): number {
	if (!result) return 0;
	try {
		const data = result as { results?: number[][] };
		if (data.results && data.results[0] && typeof data.results[0][0] === 'number') {
			return data.results[0][0];
		}
	} catch {
		return 0;
	}
	return 0;
}

function extractProviderStats(result: PostHogInsightResult | null): ProviderStats[] {
	if (!result) return [];
	try {
		const data = result as { results?: (string | number | null)[][] };
		if (data.results && Array.isArray(data.results)) {
			return data.results
				.filter((row) => row[0] && typeof row[0] === 'string')
				.map((row) => ({
					provider: row[0] as string,
					model: row[1] && typeof row[1] === 'string' && row[1] !== '' ? row[1] : null,
					count: typeof row[2] === 'number' ? row[2] : 0,
					avgLatencyMs: typeof row[3] === 'number' ? Math.round(row[3]) : 0
				}));
		}
	} catch {
		return [];
	}
	return [];
}
