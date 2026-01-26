<script lang="ts">
	import { Users, Terminal, Star } from '@lucide/svelte';

	interface ProviderStats {
		provider: string;
		model: string | null;
		count: number;
		avgLatencyMs: number;
	}

	interface Props {
		stats: {
			totalUsers: number;
			totalCommands: number;
			githubStars: number;
			topProviders: ProviderStats[];
		};
	}

	let { stats }: Props = $props();

	function formatNumber(num: number): string {
		if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
		if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
		return num.toString();
	}

	function formatLatency(ms: number): string {
		if (ms >= 1000) return (ms / 1000).toFixed(1) + 's';
		return ms + 'ms';
	}

	function formatProviderName(provider: string): string {
		const names: Record<string, string> = {
			openrouter: 'OpenRouter',
			gemini: 'Gemini',
			anthropic: 'Anthropic',
			ollama: 'Ollama',
			lmstudio: 'LM Studio',
			embedded: 'Embedded',
			local: 'Local',
			cloud: 'Cloud'
		};
		return names[provider.toLowerCase()] || provider;
	}

	function formatModelName(model: string): string {
		const parts = model.split('/');
		return parts[parts.length - 1];
	}
</script>

<section class="border-t border-border py-16 lg:py-24">
	<div class="container mx-auto px-4">
		<div class="mb-12 text-center">
			<h2 class="mb-4 text-3xl font-bold lg:text-4xl">Trusted by Developers</h2>
			<p class="mx-auto max-w-2xl text-muted-foreground">
				Join the growing community of developers who are saying hi to their shell.
			</p>
		</div>

		<!-- Stats Cards -->
		<div class="mx-auto mb-8 grid max-w-2xl gap-4 sm:grid-cols-3">
			<div
				class="rounded-xl border border-border bg-card p-6 text-center transition-all hover:border-primary/50 hover:shadow-lg"
			>
				<div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-yellow-500/10">
					<Star class="h-6 w-6 text-yellow-500" />
				</div>
				<div class="text-3xl font-bold text-foreground">{formatNumber(stats.githubStars)}</div>
				<div class="text-sm text-muted-foreground">GitHub Stars</div>
			</div>

			<div
				class="rounded-xl border border-border bg-card p-6 text-center transition-all hover:border-primary/50 hover:shadow-lg"
			>
				<div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-blue-500/10">
					<Users class="h-6 w-6 text-blue-500" />
				</div>
				<div class="text-3xl font-bold text-foreground">{formatNumber(stats.totalUsers)}</div>
				<div class="text-sm text-muted-foreground">Unique Users</div>
			</div>

			<div
				class="rounded-xl border border-border bg-card p-6 text-center transition-all hover:border-primary/50 hover:shadow-lg"
			>
				<div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-emerald-500/10">
					<Terminal class="h-6 w-6 text-emerald-500" />
				</div>
				<div class="text-3xl font-bold text-foreground">{formatNumber(stats.totalCommands)}</div>
				<div class="text-sm text-muted-foreground">Commands Generated</div>
			</div>
		</div>

		<!-- Provider Stats Table -->
		{#if stats.topProviders.length > 0}
			<div class="mx-auto max-w-2xl rounded-xl border border-border bg-card p-6">
				<h3 class="mb-4 text-center text-lg font-semibold">Popular Models</h3>
				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead>
							<tr class="border-b border-border text-left text-muted-foreground">
								<th class="pb-3 font-medium">Provider</th>
								<th class="pb-3 font-medium">Model</th>
								<th class="pb-3 font-medium text-right">Commands</th>
								<th class="pb-3 font-medium text-right">Avg Response</th>
							</tr>
						</thead>
						<tbody>
							{#each stats.topProviders as item}
								<tr class="border-b border-border/50 last:border-0">
									<td class="py-3 font-medium text-foreground">{formatProviderName(item.provider)}</td>
									<td class="py-3 text-muted-foreground">
										{#if item.model}
											<code class="rounded bg-muted px-1.5 py-0.5 text-xs">{formatModelName(item.model)}</code>
										{:else}
											<span class="text-muted-foreground/50">—</span>
										{/if}
									</td>
									<td class="py-3 text-right text-muted-foreground">{formatNumber(item.count)}</td>
									<td class="py-3 text-right text-muted-foreground">{formatLatency(item.avgLatencyMs)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		{/if}
	</div>
</section>
