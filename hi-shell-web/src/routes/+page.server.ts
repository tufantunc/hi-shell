import { fetchStats } from '$lib/server/posthog';

export async function load() {
	const stats = await fetchStats();
	return { stats };
}
