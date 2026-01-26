/**
 * Cloudflare Worker for scheduled builds
 * 
 * Setup instructions:
 * 1. Create a deploy hook in Cloudflare Pages: Settings > Builds > Add deploy hook
 * 2. Create a new Worker in Cloudflare dashboard
 * 3. Paste this code into the Worker
 * 4. Add environment variable: DEPLOY_HOOK_URL = your deploy hook URL
 * 5. Add a Cron Trigger: Settings > Triggers > Add Cron Trigger
 *    Example: 0 5 * * * (runs daily at 05:00 UTC)
 */

export default {
	async scheduled(controller, env, ctx) {
		if (!env.DEPLOY_HOOK_URL) {
			console.error('DEPLOY_HOOK_URL environment variable is not set');
			return;
		}

		try {
			const response = await fetch(env.DEPLOY_HOOK_URL, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
			});

			if (response.ok) {
				console.log('Successfully triggered build at', new Date().toISOString());
			} else {
				console.error('Failed to trigger build:', response.status, response.statusText);
			}
		} catch (error) {
			console.error('Error triggering build:', error);
		}
	},

	async fetch(request) {
		return new Response('This worker is for scheduled tasks only. Add a Cron Trigger to use it.', {
			status: 200,
			headers: { 'Content-Type': 'text/plain' },
		});
	},
};
