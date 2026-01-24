<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Component } from 'svelte';

	interface Props {
		title: string;
		description: string;
		icon: Component;
	}

	let { title, description, icon: Icon }: Props = $props();
</script>

<div
	class="rounded-xl border border-border bg-card p-6 transition-all hover:border-primary/50 hover:shadow-lg"
>
	<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
		<Icon class="h-6 w-6 text-primary" />
	</div>
	<h3 class="mb-2 text-lg font-semibold">{title}</h3>
	<p class="text-sm text-muted-foreground">{description}</p>
</div>
