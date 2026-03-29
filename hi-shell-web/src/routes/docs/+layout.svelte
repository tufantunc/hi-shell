<script lang="ts">
	import { tick } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/stores';
	import DocsSidebar from '$lib/components/DocsSidebar.svelte';
	import Header from '$lib/components/Header.svelte';
	import { Menu, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { enhanceCodeBlocks } from '$lib/code-block-enhancer';
	import './docs.css';

	let { children } = $props();
	let sidebarOpen = $state(false);
	let articleEl: HTMLElement | undefined = $state();

	afterNavigate(async () => {
		await tick();
		requestAnimationFrame(() => {
			if (articleEl) enhanceCodeBlocks(articleEl);
		});
	});
</script>

<svelte:head>
	<meta name="robots" content="index, follow" />
</svelte:head>

<div class="min-h-screen bg-background">
	<Header />

	<div class="container mx-auto px-4 py-8">
		<div class="flex gap-8">
			<!-- Mobile sidebar toggle -->
			<div class="lg:hidden fixed bottom-4 right-4 z-50">
				<Button
					size="icon"
					class="h-12 w-12 rounded-full shadow-lg"
					onclick={() => (sidebarOpen = !sidebarOpen)}
				>
					{#if sidebarOpen}
						<X class="h-5 w-5" />
					{:else}
						<Menu class="h-5 w-5" />
					{/if}
				</Button>
			</div>

			<!-- Sidebar - desktop -->
			<aside class="hidden w-64 shrink-0 lg:block">
				<DocsSidebar />
			</aside>

			<!-- Sidebar - mobile overlay -->
			{#if sidebarOpen}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="fixed inset-0 z-40 bg-black/50 lg:hidden" onclick={() => (sidebarOpen = false)} role="presentation"></div>
				<aside class="fixed left-0 top-0 z-50 h-full w-72 bg-background p-6 shadow-xl lg:hidden">
					<DocsSidebar />
				</aside>
			{/if}

			<!-- Main content -->
			<main class="min-w-0 flex-1">
			{#key $page.url.pathname}
				<article
					bind:this={articleEl}
					class="prose prose-slate max-w-none dark:prose-invert
					prose-headings:scroll-mt-20
					prose-a:text-primary prose-a:no-underline hover:prose-a:underline
					prose-pre:bg-zinc-900 prose-pre:text-zinc-100
					prose-code:text-primary prose-code:before:content-none prose-code:after:content-none
					prose-img:rounded-lg">
					{@render children()}
				</article>
			{/key}
		</main>
		</div>
	</div>
</div>
