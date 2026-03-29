<script lang="ts">
	import Header from '$lib/components/Header.svelte';
	import PlaygroundTerminal from '$lib/components/PlaygroundTerminal.svelte';
	import { Button } from '$lib/components/ui/button';
	import { ArrowRight, Github, Info } from '@lucide/svelte';
</script>

<svelte:head>
	<title>Playground</title>
	<meta name="description" content="Try hi-shell in your browser. Type natural language and see AI-generated shell commands in this interactive demo." />
	<meta property="og:title" content="Playground - hi-shell" />
	<meta property="og:description" content="Try hi-shell in your browser with this interactive demo." />
	<meta name="robots" content="noindex" />
	<link rel="canonical" href="https://hi-shell.dev/playground" />
</svelte:head>

<div class="min-h-screen bg-background">
	<Header />

	<div class="container mx-auto px-4 py-12">
		<div class="mx-auto max-w-3xl">
			<div class="mb-8 text-center">
				<h1 class="mb-4 text-4xl font-bold">Try hi-shell</h1>
				<p class="text-lg text-muted-foreground">
					Experience how hi-shell translates natural language into shell commands.
					This is a simulated demo — no AI or API calls involved.
				</p>
			</div>

			<div class="mb-6 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-900 dark:bg-blue-950/30">
				<div class="flex items-start gap-3">
					<Info class="mt-0.5 h-5 w-5 shrink-0 text-blue-600 dark:text-blue-400" />
					<div>
						<p class="text-sm text-blue-800 dark:text-blue-200">
							This playground uses <strong>predefined responses</strong> for common commands.
							It does not connect to any AI service. For the real experience,
							<a href="/#installation" class="font-medium underline">install hi-shell</a>.
						</p>
						<p class="mt-2 text-sm text-blue-700 dark:text-blue-300">
							Try: <em>"list files"</em>, <em>"docker containers"</em>, <em>"git status"</em>,
							<em>"find large files"</em>, <em>"delete log files"</em>, <em>"kill process"</em>
						</p>
					</div>
				</div>
			</div>

			<PlaygroundTerminal />

			<div class="mt-8 text-center">
				<h3 class="mb-2 text-xl font-semibold">Ready for the real thing?</h3>
				<p class="mb-4 text-muted-foreground">
					Install hi-shell for AI-powered command generation with your choice of LLM provider.
				</p>
				<div class="flex flex-wrap justify-center gap-4">
					<Button size="lg" href="/#installation">
						Install hi-shell
						<ArrowRight class="ml-2 h-4 w-4" />
					</Button>
					<Button variant="outline" size="lg" href="https://github.com/tufantunc/hi-shell">
						<Github class="mr-2 h-4 w-4" />
						View Source
					</Button>
				</div>
			</div>
		</div>
	</div>
</div>
