<script lang="ts">
	import Header from '$lib/components/Header.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Tag, Calendar, ExternalLink } from '@lucide/svelte';

	let { data } = $props();

	function formatDate(dateStr: string): string {
		try {
			return new Date(dateStr).toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'long',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	function renderMarkdown(text: string): string {
		return text
			.replace(/^### (.+)$/gm, '<h3 class="text-lg font-semibold mt-4 mb-2">$1</h3>')
			.replace(/^## (.+)$/gm, '<h2 class="text-xl font-semibold mt-6 mb-3">$1</h2>')
			.replace(/^# (.+)$/gm, '<h1 class="text-2xl font-bold mt-8 mb-4">$1</h1>')
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/`([^`]+)`/g, '<code class="rounded bg-muted px-1.5 py-0.5 text-sm font-mono">$1</code>')
			.replace(/^- (.+)$/gm, '<li class="ml-4 list-disc text-muted-foreground">$1</li>')
			.replace(/\n\n/g, '<br/><br/>')
			.replace(/\n/g, '<br/>');
	}
</script>

<svelte:head>
	<title>Changelog</title>
	<meta name="description" content="Release notes and changelog for hi-shell. See what's new in each version." />
	<meta property="og:title" content="Changelog - hi-shell" />
	<meta property="og:description" content="Release notes and changelog for hi-shell." />
	<link rel="canonical" href="https://hi-shell.dev/changelog" />
</svelte:head>

<div class="min-h-screen bg-background">
	<Header />

	<div class="container mx-auto px-4 py-12">
		<div class="mx-auto max-w-3xl">
			<div class="mb-12 text-center">
				<h1 class="mb-4 text-4xl font-bold">Changelog</h1>
				<p class="text-lg text-muted-foreground">
					What's new in hi-shell. All releases from GitHub.
				</p>
			</div>

			{#if data.releases.length === 0}
				<div class="rounded-lg border border-border bg-card p-8 text-center">
					<Tag class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
					<h3 class="mb-2 text-lg font-semibold">No releases yet</h3>
					<p class="text-muted-foreground">
						Check back soon or view the
						<a
							href="https://github.com/tufantunc/hi-shell/releases"
							class="text-primary underline"
						>
							GitHub releases page
						</a>.
					</p>
				</div>
			{:else}
				<div class="relative space-y-8">
					<!-- Timeline line -->
					<div class="absolute left-[19px] top-0 bottom-0 w-px bg-border"></div>

					{#each data.releases as release, i}
						<div class="relative pl-12">
							<!-- Timeline dot -->
							<div class="absolute left-[12px] top-6 h-4 w-4 rounded-full {i === 0 ? 'bg-primary ring-4 ring-primary/20' : 'bg-muted-foreground/30'}"></div>

							<div class="rounded-lg border border-border bg-card p-6">
								<div class="mb-4 flex flex-wrap items-center gap-3">
									<h2 class="text-xl font-bold">{release.name}</h2>
									{#if i === 0}
										<Badge>Latest</Badge>
									{/if}
									{#if release.prerelease}
										<Badge variant="secondary">Pre-release</Badge>
									{/if}
								</div>

								<div class="mb-4 flex flex-wrap items-center gap-4 text-sm text-muted-foreground">
									<span class="flex items-center gap-1">
										<Tag class="h-4 w-4" />
										{release.tag}
									</span>
									<span class="flex items-center gap-1">
										<Calendar class="h-4 w-4" />
										{formatDate(release.date)}
									</span>
									<a
										href={release.url}
										target="_blank"
										rel="noopener noreferrer"
										class="flex items-center gap-1 text-primary hover:underline"
									>
										<ExternalLink class="h-4 w-4" />
										View on GitHub
									</a>
								</div>

								{#if release.body}
									<div class="prose prose-sm prose-slate max-w-none dark:prose-invert">
										{@html renderMarkdown(release.body)}
									</div>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
