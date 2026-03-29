<script lang="ts">
	import { page } from '$app/stores';
	import { Book, ChevronRight, Cpu, Server, Cloud, Settings, HelpCircle, AlertTriangle, Home } from '@lucide/svelte';

	interface NavItem {
		label: string;
		href: string;
		icon: any;
	}

	interface NavGroup {
		title: string;
		items: NavItem[];
	}

	const groups: NavGroup[] = [
		{
			title: 'Getting Started',
			items: [
				{ label: 'Overview', href: '/docs', icon: Home },
				{ label: 'Getting Started', href: '/docs/getting-started', icon: Book }
			]
		},
		{
			title: 'Providers',
			items: [
				{ label: 'Embedded Models', href: '/docs/providers/embedded', icon: Cpu },
				{ label: 'Ollama', href: '/docs/providers/ollama', icon: Server },
				{ label: 'LM Studio', href: '/docs/providers/lm-studio', icon: Server },
				{ label: 'OpenRouter', href: '/docs/providers/openrouter', icon: Cloud },
				{ label: 'Anthropic', href: '/docs/providers/anthropic', icon: Cloud },
				{ label: 'OpenAI', href: '/docs/providers/openai', icon: Cloud }
			]
		},
		{
			title: 'Reference',
			items: [
				{ label: 'Configuration', href: '/docs/configuration', icon: Settings },
				{ label: 'Troubleshooting', href: '/docs/troubleshooting', icon: AlertTriangle },
				{ label: 'FAQ', href: '/docs/faq', icon: HelpCircle }
			]
		}
	];
</script>

<nav class="docs-sidebar">
	{#each groups as group}
		<div class="mb-6">
			<h3 class="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
				{group.title}
			</h3>
			<ul class="space-y-1">
				{#each group.items as item}
					{@const isActive = $page.url.pathname === item.href}
					<li>
						<a
							href={item.href}
							class="group flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors {isActive
								? 'bg-primary/10 font-medium text-primary'
								: 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
						>
							<item.icon class="h-4 w-4 shrink-0" />
							<span class="flex-1">{item.label}</span>
							{#if isActive}
								<ChevronRight class="h-3 w-3" />
							{/if}
						</a>
					</li>
				{/each}
			</ul>
		</div>
	{/each}
</nav>

<style>
	.docs-sidebar {
		position: sticky;
		top: 5rem;
		max-height: calc(100vh - 6rem);
		overflow-y: auto;
	}
</style>
