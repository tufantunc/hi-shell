interface GitHubRelease {
	tag_name: string;
	name: string;
	body: string;
	published_at: string;
	prerelease: boolean;
	html_url: string;
}

export async function load({ fetch }) {
	try {
		const response = await fetch(
			'https://api.github.com/repos/tufantunc/hi-shell/releases?per_page=20',
			{
				headers: { Accept: 'application/vnd.github.v3+json' }
			}
		);

		if (!response.ok) {
			return { releases: [] };
		}

		const releases: GitHubRelease[] = await response.json();

		return {
			releases: releases.map((r) => ({
				tag: r.tag_name,
				name: r.name || r.tag_name,
				body: r.body || '',
				date: r.published_at,
				prerelease: r.prerelease,
				url: r.html_url
			}))
		};
	} catch {
		return { releases: [] };
	}
}
