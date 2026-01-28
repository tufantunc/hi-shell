<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import { Github, Menu, X } from '@lucide/svelte';

	let mobileMenuOpen = $state(false);

	function toggleMenu() {
		mobileMenuOpen = !mobileMenuOpen;
	}

	function closeMenu() {
		mobileMenuOpen = false;
	}
</script>

<header class="sticky top-0 z-50 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
	<div class="container mx-auto flex items-center justify-between px-4 py-4">
		<a href="/" class="flex items-center gap-2 text-xl font-bold">
			<span class="text-2xl">🐚</span>
			<span>hi-shell</span>
		</a>

		<!-- Desktop Navigation -->
		<nav class="hidden items-center gap-4 md:flex">
			<a href="#features" class="text-sm text-muted-foreground hover:text-foreground">Features</a>
			<a href="#installation" class="text-sm text-muted-foreground hover:text-foreground">Installation</a>
			<a href="#usage" class="text-sm text-muted-foreground hover:text-foreground">Usage</a>
			<a href="#privacy-security" class="text-sm text-muted-foreground hover:text-foreground">Privacy</a>
			<ThemeToggle />
			<Button variant="outline" size="sm" href="https://github.com/tufantunc/hi-shell">
				<Github class="mr-2 h-4 w-4" />
				GitHub
			</Button>
		</nav>

		<!-- Mobile Menu Button -->
		<div class="flex items-center gap-2 md:hidden">
			<ThemeToggle />
			<Button variant="ghost" size="icon" onclick={toggleMenu} aria-label="Toggle menu">
				{#if mobileMenuOpen}
					<X class="h-5 w-5" />
				{:else}
					<Menu class="h-5 w-5" />
				{/if}
			</Button>
		</div>
	</div>

	<!-- Mobile Navigation -->
	{#if mobileMenuOpen}
		<nav class="border-t border-border bg-background px-4 py-4 md:hidden">
			<div class="flex flex-col gap-4">
				<a
					href="#features"
					class="text-sm text-muted-foreground hover:text-foreground"
					onclick={closeMenu}
				>
					Features
				</a>
				<a
					href="#installation"
					class="text-sm text-muted-foreground hover:text-foreground"
					onclick={closeMenu}
				>
					Installation
				</a>
				<a
					href="#usage"
					class="text-sm text-muted-foreground hover:text-foreground"
					onclick={closeMenu}
				>
					Usage
				</a>
				<a
					href="#privacy-security"
					class="text-sm text-muted-foreground hover:text-foreground"
					onclick={closeMenu}
				>
					Privacy
				</a>
				<Button variant="outline" size="sm" href="https://github.com/tufantunc/hi-shell" class="w-fit">
					<Github class="mr-2 h-4 w-4" />
					GitHub
				</Button>
			</div>
		</nav>
	{/if}
</header>
