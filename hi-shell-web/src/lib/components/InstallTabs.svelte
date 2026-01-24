<script lang="ts">
	import { onMount } from 'svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import { Copy, Check } from '@lucide/svelte';

	type Platform = 'macos' | 'linux' | 'windows';

	const installMethods = {
		quick: {
			label: '⚡ Quick Install',
			platforms: ['macos', 'linux'] as Platform[],
			command: 'curl -sSL https://raw.githubusercontent.com/tufantunc/hi-shell/main/install.sh | bash'
		},
		homebrew: {
			label: '🍏 Homebrew',
			platforms: ['macos', 'linux'] as Platform[],
			command: 'brew tap tufantunc/tap && brew install hi-shell'
		},
		scoop: {
			label: '🪟 Scoop',
			platforms: ['windows'] as Platform[],
			command: 'scoop bucket add hi-shell https://github.com/tufantunc/scoop-bucket && scoop install hi-shell'
		},
		cargo: {
			label: '🦀 Cargo',
			platforms: ['macos', 'linux', 'windows'] as Platform[],
			command: 'cargo install hi-shell'
		}
	};

	let detectedPlatform = $state<Platform>('macos');
	let activeTab = $state('quick');
	let copiedStates = $state<Record<string, boolean>>({});

	function detectPlatform(): Platform {
		if (typeof navigator === 'undefined') return 'macos';
		const ua = navigator.userAgent.toLowerCase();
		if (ua.includes('win')) return 'windows';
		if (ua.includes('linux')) return 'linux';
		return 'macos';
	}

	function getDefaultTab(platform: Platform): string {
		if (platform === 'windows') return 'scoop';
		return 'quick';
	}

	async function copyToClipboard(key: string, command: string) {
		await navigator.clipboard.writeText(command);
		copiedStates[key] = true;
		setTimeout(() => {
			copiedStates[key] = false;
		}, 2000);
	}

	onMount(() => {
		detectedPlatform = detectPlatform();
		activeTab = getDefaultTab(detectedPlatform);
	});

	const allMethods = Object.entries(installMethods);

	function isRecommended(platforms: Platform[]): boolean {
		return platforms.includes(detectedPlatform);
	}
</script>

<div class="w-full">
	<div class="mb-3 flex items-center gap-2 text-sm text-muted-foreground">
		<span>Detected:</span>
		<span class="rounded bg-secondary px-2 py-0.5 font-medium text-secondary-foreground">
			{detectedPlatform === 'macos' ? '🍏 macOS' : detectedPlatform === 'windows' ? '🪟 Windows' : '🐧 Linux'}
		</span>
	</div>

	<Tabs.Root bind:value={activeTab} class="w-full">
		<Tabs.List class="mb-4 flex flex-wrap gap-1">
			{#each allMethods as [key, method]}
				<Tabs.Trigger value={key} class="text-sm">
					{method.label}
					{#if isRecommended(method.platforms)}
						<span class="ml-1 text-xs text-emerald-500">✓</span>
					{/if}
				</Tabs.Trigger>
			{/each}
		</Tabs.List>

		{#each allMethods as [key, method]}
			<Tabs.Content value={key}>
				<div class="mb-3 flex items-center gap-2 text-sm">
					<span class="text-muted-foreground">Supported:</span>
					{#each method.platforms as platform}
						<span class="rounded bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground">
							{platform === 'macos' ? '🍏 macOS' : platform === 'windows' ? '🪟 Windows' : '🐧 Linux'}
						</span>
					{/each}
				</div>
				{#if !isRecommended(method.platforms)}
					<div class="mb-2 text-sm text-yellow-600 dark:text-yellow-400">
						⚠️ This method is not available for your system ({detectedPlatform === 'macos' ? 'macOS' : detectedPlatform === 'windows' ? 'Windows' : 'Linux'})
					</div>
				{/if}
				<div class="group relative">
					<pre
						class="overflow-x-auto rounded-lg bg-zinc-900 p-4 text-sm text-zinc-100"><code>{method.command}</code></pre>
					<button
						onclick={() => copyToClipboard(key, method.command)}
						class="absolute right-2 top-2 rounded-md bg-zinc-700 p-2 opacity-0 transition-opacity hover:bg-zinc-600 group-hover:opacity-100"
						aria-label="Copy to clipboard"
					>
						{#if copiedStates[key]}
							<Check class="h-4 w-4 text-green-400" />
						{:else}
							<Copy class="h-4 w-4 text-zinc-300" />
						{/if}
					</button>
				</div>
			</Tabs.Content>
		{/each}
	</Tabs.Root>
</div>
