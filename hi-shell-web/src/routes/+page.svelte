<script lang="ts">
	import InteractiveTerminal from '$lib/components/InteractiveTerminal.svelte';
	import ExampleTerminal from '$lib/components/ExampleTerminal.svelte';
	import InstallTabs from '$lib/components/InstallTabs.svelte';
	import FeatureCard from '$lib/components/FeatureCard.svelte';
	import Header from '$lib/components/Header.svelte';
	import StatsSection from '$lib/components/StatsSection.svelte';

	let { data } = $props();

	const oneShotDemo = [
		{ type: 'input' as const, text: 'find all png files larger than 1mb', delay: 45 },
		{ type: 'generating' as const, text: '' },
		{ type: 'output' as const, text: 'find . -name "*.png" -size +1M', delay: 300 },
		{ type: 'result' as const, text: './images/photo.png\n./assets/banner.png\n./uploads/hero.png', delay: 2500 }
	];

	const replDemo = [
		{ type: 'input' as const, text: 'list all docker containers', delay: 40 },
		{ type: 'generating' as const, text: '' },
		{ type: 'output' as const, text: 'docker ps -a', delay: 300 },
		{ type: 'result' as const, text: 'CONTAINER ID  IMAGE   STATUS\na1b2c3d4e5f6  nginx   Up 2 hours\nf6e5d4c3b2a1  redis   Exited', delay: 1000 },
		{ type: 'input' as const, text: 'now show only running ones', delay: 45 },
		{ type: 'generating' as const, text: '' },
		{ type: 'output' as const, text: 'docker ps', delay: 300 },
		{ type: 'result' as const, text: 'CONTAINER ID  IMAGE   STATUS\na1b2c3d4e5f6  nginx   Up 2 hours', delay: 1000 },
		{ type: 'input' as const, text: 'stop all of them', delay: 50 },
		{ type: 'generating' as const, text: '' },
		{ type: 'output' as const, text: 'docker stop $(docker ps -q)', delay: 300 },
		{ type: 'result' as const, text: 'a1b2c3d4e5f6\nStopped 1 container', delay: 2000 }
	];

	const safetyDemo = [
		{ type: 'input' as const, text: 'delete all log files recursively', delay: 45 },
		{ type: 'generating' as const, text: '' },
		{ type: 'warning' as const, text: '⚠ Warning: This command is potentially dangerous', delay: 200 },
		{ type: 'output' as const, text: 'find . -name "*.log" -type f -delete', delay: 200 },
		{ type: 'confirm' as const, text: '' }
	];
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Cpu,
		Cloud,
		Server,
		Shield,
		Terminal,
		Zap,
		Github,
		ArrowRight,
		Sparkles,
		ShieldCheck,
		Lock,
		Users,
		Code
	} from '@lucide/svelte';
</script>

<svelte:head>
	<title>hi-shell - AI-Powered Terminal Assistant</title>
	<meta
		name="description"
		content="An intelligent terminal assistant that translates your natural language descriptions into executable bash commands."
	/>
</svelte:head>

<div class="min-h-screen bg-background">
	<Header />

	<!-- Hero Section -->
	<section class="container mx-auto px-4 py-16 lg:py-24">
		<div class="grid items-center gap-12 lg:grid-cols-2">
			<div class="space-y-6">
				<Badge variant="secondary" class="mb-4">
					<Sparkles class="mr-1 h-3 w-3" />
					AI-Powered Terminal Assistant
				</Badge>
				<h1 class="text-4xl font-bold tracking-tight lg:text-5xl xl:text-6xl">
					Say <span class="text-primary">hi</span> to your shell
				</h1>
				<p class="text-lg text-muted-foreground lg:text-xl">
					Bridge the gap between "what I want to do" and "how do I write that command?" with an
					intelligent terminal assistant that translates natural language into executable bash
					commands.
				</p>

				<div class="flex flex-wrap gap-4">
					<Button size="lg" href="#installation">
						Get Started
						<ArrowRight class="ml-2 h-4 w-4" />
					</Button>
					<Button variant="outline" size="lg" href="https://github.com/tufantunc/hi-shell">
						<Github class="mr-2 h-4 w-4" />
						View on GitHub
					</Button>
				</div>

				<div class="flex flex-wrap items-center gap-4 pt-4 text-sm text-muted-foreground">
					<div class="flex items-center gap-1">
						<Shield class="h-4 w-4 text-green-500" />
						<span>Safe by default</span>
					</div>
					<div class="flex items-center gap-1">
						<Cpu class="h-4 w-4 text-blue-500" />
						<span>Runs locally</span>
					</div>
					<div class="flex items-center gap-1">
						<Zap class="h-4 w-4 text-yellow-500" />
						<span>Lightning fast</span>
					</div>
				</div>
			</div>

			<div class="lg:pl-8">
				<InteractiveTerminal />
			</div>
		</div>
	</section>

	<!-- Stats Section -->
	{#if data.stats.totalUsers > 0}
		<StatsSection stats={data.stats} />
	{/if}

	<!-- Features Section -->
	<section id="features" class="border-t border-border bg-muted/30 py-16 lg:py-24">
		<div class="container mx-auto px-4">
			<div class="mb-12 text-center">
				<h2 class="mb-4 text-3xl font-bold lg:text-4xl">Powerful Features</h2>
				<p class="mx-auto max-w-2xl text-muted-foreground">
					Whether you're a terminal veteran or a newcomer, hi-shell provides a fast, AI-powered way
					to generate and execute commands safely.
				</p>
			</div>

			<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
				<FeatureCard
					title="Embedded Models"
					description="Run models locally using candle with hardware acceleration (Metal/CUDA). Supports Llama, Phi-3, and Qwen2 architectures."
					icon={Cpu}
				/>
				<FeatureCard
					title="Local LLM Support"
					description="Connect to your own Ollama or LM Studio instance for complete privacy and control."
					icon={Server}
				/>
				<FeatureCard
					title="Cloud Integration"
					description="Seamless integration with OpenRouter, Gemini, and Anthropic for powerful cloud-based models."
					icon={Cloud}
				/>
				<FeatureCard
					title="Interactive REPL"
					description="A dedicated shell environment for continuous assistance and iterative command building."
					icon={Terminal}
				/>
				<FeatureCard
					title="Safety First"
					description="Dangerous commands are flagged, and confirmation is required before execution. Your system stays safe."
					icon={Shield}
				/>
				<FeatureCard
					title="Lightning Fast"
					description="Optimized for speed with hardware acceleration support. Get your commands in milliseconds."
					icon={Zap}
				/>
			</div>
		</div>
	</section>

	<!-- Installation Section -->
	<section id="installation" class="py-16 lg:py-24">
		<div class="container mx-auto px-4">
			<div class="mb-12 text-center">
				<h2 class="mb-4 text-3xl font-bold lg:text-4xl">Installation</h2>
				<p class="mx-auto max-w-2xl text-muted-foreground">
					Choose your preferred installation method. We detect your operating system automatically.
				</p>
			</div>

			<div class="mx-auto max-w-2xl">
				<InstallTabs />

				<div class="mt-8 rounded-lg border border-border bg-card p-6">
					<h3 class="mb-3 font-semibold">After Installation</h3>
					<p class="mb-4 text-sm text-muted-foreground">
						Run the initialization command to set up your preferred LLM provider:
					</p>
					<pre
						class="overflow-x-auto rounded-lg bg-zinc-900 p-4 text-sm text-zinc-100"><code>hi-shell --init</code></pre>
				</div>
			</div>
		</div>
	</section>

	<!-- Usage Section -->
	<section id="usage" class="border-t border-border bg-muted/30 py-16 lg:py-24">
		<div class="container mx-auto px-4">
			<div class="mb-12 text-center">
				<h2 class="mb-4 text-3xl font-bold lg:text-4xl">Usage Examples</h2>
				<p class="mx-auto max-w-2xl text-muted-foreground">
					Just prefix your natural language request with <code class="rounded bg-zinc-800 px-2 py-1 text-emerald-400">hi-shell</code> and let
					the magic happen.
				</p>
			</div>

			<div class="mx-auto max-w-5xl space-y-12">
				<!-- One-shot Mode -->
				<div class="grid items-center gap-8 lg:grid-cols-2">
					<div>
						<h3 class="mb-3 text-xl font-semibold">One-shot Mode</h3>
						<p class="text-muted-foreground">
							Get quick answers directly from your command line. Just describe what you want to do
							in natural language and get the exact command you need.
						</p>
					</div>
					<ExampleTerminal lines={oneShotDemo} title="One-shot Mode" />
				</div>

				<!-- Interactive REPL Mode -->
				<div class="grid items-center gap-8 lg:grid-cols-2">
					<div class="lg:order-2">
						<h3 class="mb-3 text-xl font-semibold">Interactive REPL Mode</h3>
						<p class="text-muted-foreground">
							Start a dedicated shell environment for continuous assistance. The context is
							preserved between commands, so you can refine your requests naturally like a
							conversation.
						</p>
					</div>
					<div class="lg:order-1">
						<ExampleTerminal lines={replDemo} title="Interactive Mode" />
					</div>
				</div>

				<!-- Safety Features -->
				<div class="grid items-center gap-8 lg:grid-cols-2">
					<div>
						<h3 class="mb-3 text-xl font-semibold">Safety First</h3>
						<p class="text-muted-foreground">
							Dangerous commands are automatically detected and flagged with a warning.
							Confirmation is always required before execution, keeping your system safe.
						</p>
					</div>
					<ExampleTerminal lines={safetyDemo} title="Safety Features" />
				</div>
			</div>
		</div>
	</section>

	<!-- Privacy & Security Section -->
	<section id="privacy-security" class="border-t border-border bg-muted/30 py-16 lg:py-24">
		<div class="container mx-auto px-4">
			<div class="mb-12 text-center">
				<Badge variant="secondary" class="mb-4">
					<ShieldCheck class="mr-1 h-3 w-3" />
					Open Source & Privacy First
				</Badge>
				<h2 class="mb-4 text-3xl font-bold lg:text-4xl">Built for Privacy & Openness</h2>
				<p class="mx-auto max-w-2xl text-muted-foreground">
					Your data and privacy are our top priority. Everything is open source, transparent, and fully under
					your control.
				</p>
			</div>

			<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
				<FeatureCard
					title="Open Source MIT License"
					description="All source code is fully available under MIT license. Inspect, modify, distribute, and improve it freely."
					icon={Github}
				/>
				<FeatureCard
					title="100% Local Operation"
					description="Connect to your own Ollama or LM Studio instance. Run completely offline without any internet connection."
					icon={Lock}
				/>
				<FeatureCard
					title="Embedded Models"
					description="Run Llama, Phi-3, and Qwen2 models locally on your own hardware with Metal/CUDA acceleration."
					icon={Cpu}
				/>
				<FeatureCard
					title="No User Data Tracking"
					description="We never track your commands, prompts, or outputs. Only anonymous system information is collected (opt-in)."
					icon={Shield}
				/>
				<FeatureCard
					title="Optional Telemetry"
					description="Completely opt-in anonymous usage statistics. No personal data is ever collected. Disable anytime."
					icon={ShieldCheck}
				/>
				<FeatureCard
					title="Transparent Codebase"
					description="Fully inspectable code. Every commit, change, and discussion is visible on GitHub."
					icon={Code}
				/>
			</div>

			<div class="mt-12 grid gap-6 md:grid-cols-2">
				<div class="rounded-lg border border-border bg-card p-6">
					<h3 class="mb-2 flex items-center gap-2 text-lg font-semibold">
						<Users class="h-5 w-5 text-primary" />
						Community Audited
					</h3>
					<p class="text-sm text-muted-foreground">
						Active community with issues and PRs ensures continuous oversight and improvement. Security
						vulnerabilities are quickly identified and fixed.
					</p>
				</div>
				<div class="rounded-lg border border-border bg-card p-6">
					<h3 class="mb-2 flex items-center gap-2 text-lg font-semibold">
						<ShieldCheck class="h-5 w-5 text-primary" />
						Safety Built-In
					</h3>
					<p class="text-sm text-muted-foreground">
						Dangerous commands are automatically detected and flagged. Confirmation is always required before
						execution. Your system stays protected by default.
					</p>
				</div>
			</div>

			<div class="mt-12 rounded-lg bg-card p-6 text-center">
				<h3 class="mb-2 text-xl font-semibold">Total Control in Your Hands</h3>
				<p class="mb-4 text-muted-foreground">
					Inspect the code, run it locally, contribute improvements. Everything is under your control.
				</p>
				<div class="flex flex-wrap justify-center gap-4">
					<Button href="https://github.com/tufantunc/hi-shell">
						<Github class="mr-2 h-4 w-4" />
						Explore Source Code
					</Button>
					<Button variant="outline" href="#installation">
						Try It Now
					</Button>
				</div>
			</div>
		</div>
	</section>

	<!-- CTA Section -->
	<section>
		<div class="container mx-auto text-center">
			<div class="mx-auto max-w-2xl">
				<img
					src="/small-hermit-crab-mascot.png"
					alt="hi-shell mascot"
					class="mx-auto h-64 w-64 object-contain lg:h-128 lg:w-128"
				/>
				<h2 class="mb-4 text-3xl font-bold lg:text-4xl">Ready to say hi?</h2>
				<p class="mb-8 text-muted-foreground">
					Stop searching for commands. Start describing what you want to do.
				</p>
				<div class="flex flex-wrap justify-center gap-4 mb-16">
					<Button size="lg" href="#installation">
						Install hi-shell
						<ArrowRight class="ml-2 h-4 w-4" />
					</Button>
					<Button variant="outline" size="lg" href="https://github.com/tufantunc/hi-shell">
						<Github class="mr-2 h-4 w-4" />
						Star on GitHub
					</Button>
				</div>
			</div>
		</div>
	</section>

	<!-- Footer -->
	<footer class="border-t border-border py-8">
		<div class="container mx-auto px-4">
			<div class="flex flex-col items-center justify-between gap-4 md:flex-row">
				<div class="flex items-center gap-2">
					<span class="text-xl">🐚</span>
					<span class="font-semibold">hi-shell</span>
				</div>
				<p class="text-sm text-muted-foreground">
					Released under the <a
						href="https://github.com/tufantunc/hi-shell/blob/main/LICENSE"
						class="underline hover:text-foreground">MIT License</a
					>
				</p>
				<div class="flex items-center gap-4">
					<a
						href="https://github.com/tufantunc/hi-shell"
						class="text-muted-foreground hover:text-foreground"
					>
						<Github class="h-5 w-5" />
					</a>
				</div>
			</div>
		</div>
	</footer>
</div>
