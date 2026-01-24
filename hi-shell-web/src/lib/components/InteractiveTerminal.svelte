<script lang="ts">
	import { onMount } from 'svelte';

	interface DemoStep {
		type: 'input' | 'generating' | 'output' | 'warning' | 'confirm' | 'result';
		text: string;
		delay: number;
	}

	const demos: DemoStep[][] = [
		[
			{ type: 'input', text: 'find all png files larger than 1mb', delay: 50 },
			{ type: 'generating', text: '', delay: 500 },
			{ type: 'output', text: 'find . -name "*.png" -size +1M', delay: 300 },
			{ type: 'result', text: './images/photo.png\n./assets/banner.png\n./uploads/hero.png', delay: 2000 }
		],
		[
			{ type: 'input', text: 'show disk usage sorted by size', delay: 50 },
			{ type: 'generating', text: '', delay: 450 },
			{ type: 'output', text: 'du -sh * | sort -rh', delay: 300 },
			{ type: 'result', text: '4.2G\tnode_modules\n1.1G\tdist\n256M\tsrc', delay: 2000 }
		],
		[
			{ type: 'input', text: 'delete all log files', delay: 50 },
			{ type: 'generating', text: '', delay: 500 },
			{ type: 'warning', text: '⚠ Warning: This command is potentially dangerous', delay: 300 },
			{ type: 'output', text: 'find . -name "*.log" -type f -delete', delay: 300 },
			{ type: 'confirm', text: '', delay: 2500 }
		],
		[
			{ type: 'input', text: 'count lines of code in src folder', delay: 45 },
			{ type: 'generating', text: '', delay: 600 },
			{ type: 'output', text: 'find src -name "*.ts" | xargs wc -l', delay: 300 },
			{ type: 'result', text: '  142 src/main.ts\n  89 src/utils.ts\n  231 total', delay: 2000 }
		]
	];

	let currentDemo = $state(0);
	let typedText = $state('');
	let commandText = $state('');
	let phase = $state<'idle' | 'typing' | 'generating' | 'output' | 'warning' | 'confirm' | 'result'>('idle');
	let showCursor = $state(true);
	let generatingDots = $state('');
	let warningText = $state('');
	let outputText = $state('');
	let resultText = $state('');

	function sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	async function typeText(text: string, delay: number): Promise<void> {
		typedText = '';
		for (let i = 0; i < text.length; i++) {
			typedText += text[i];
			await sleep(delay);
		}
	}

	async function animateGenerating(duration: number): Promise<void> {
		const startTime = Date.now();
		while (Date.now() - startTime < duration) {
			generatingDots = '.';
			await sleep(150);
			generatingDots = '..';
			await sleep(150);
			generatingDots = '...';
			await sleep(150);
			generatingDots = '';
		}
	}

	async function runDemo(): Promise<void> {
		while (true) {
			const demo = demos[currentDemo];
			warningText = '';
			outputText = '';
			resultText = '';

			for (const step of demo) {
				if (step.type === 'input') {
					phase = 'typing';
					commandText = step.text;
					await typeText(step.text, step.delay);
					await sleep(300);
				} else if (step.type === 'generating') {
					phase = 'generating';
					await animateGenerating(step.delay);
				} else if (step.type === 'warning') {
					phase = 'warning';
					warningText = step.text;
					await sleep(step.delay);
				} else if (step.type === 'output') {
					phase = 'output';
					outputText = step.text;
					await sleep(step.delay);
				} else if (step.type === 'confirm') {
					phase = 'confirm';
					await sleep(step.delay);
				} else if (step.type === 'result') {
					phase = 'result';
					resultText = step.text;
					await sleep(step.delay);
				}
			}

			await sleep(1000);
			currentDemo = (currentDemo + 1) % demos.length;
		}
	}

	onMount(() => {
		const cursorInterval = setInterval(() => {
			showCursor = !showCursor;
		}, 530);

		runDemo();

		return () => clearInterval(cursorInterval);
	});
</script>

<div class="overflow-hidden rounded-lg border border-zinc-700 bg-zinc-900 shadow-2xl">
	<div class="flex items-center gap-2 border-b border-zinc-700 bg-zinc-800 px-4 py-3">
		<div class="h-3 w-3 rounded-full bg-red-500"></div>
		<div class="h-3 w-3 rounded-full bg-yellow-500"></div>
		<div class="h-3 w-3 rounded-full bg-green-500"></div>
		<span class="ml-2 text-sm text-zinc-400">hi-shell</span>
	</div>

	<div class="h-56 p-4 font-mono text-sm">
		<!-- Command line -->
		<div class="flex items-start gap-2">
			<span class="text-green-400">❯</span>
			<span class="text-cyan-400">hi-shell</span>
			{#if phase === 'typing'}
				<span class="text-zinc-100">{typedText}</span>
				<span class="inline-block w-2 {showCursor ? 'bg-zinc-100' : ''}">&nbsp;</span>
			{:else}
				<span class="text-zinc-400">{commandText}</span>
			{/if}
		</div>

		<!-- Generating indicator -->
		{#if phase === 'generating'}
			<div class="mt-3 flex items-center gap-2">
				<div
					class="h-4 w-4 animate-spin rounded-full border-2 border-emerald-400 border-t-transparent"
				></div>
				<span class="text-emerald-400">Generating{generatingDots}</span>
			</div>
		{/if}

		<!-- Warning (for dangerous commands) -->
		{#if warningText && (phase === 'warning' || phase === 'output' || phase === 'confirm')}
			<div class="mt-2 text-yellow-400">{warningText}</div>
		{/if}

		<!-- Output command -->
		{#if outputText && (phase === 'output' || phase === 'confirm' || phase === 'result')}
			<div class="mt-2">
				<div class="rounded bg-zinc-800 px-3 py-2">
					<code class="text-emerald-400">{outputText}</code>
				</div>
			</div>
		{/if}

		<!-- Confirm prompt (only for dangerous commands) -->
		{#if phase === 'confirm'}
			<div class="mt-2 text-zinc-400">
				Execute? [<span class="text-emerald-400">y</span>/<span class="text-zinc-300">N</span>]
				<span class="inline-block w-2 {showCursor ? 'bg-zinc-100' : ''}">&nbsp;</span>
			</div>
		{/if}

		<!-- Result output (for safe commands that run immediately) -->
		{#if phase === 'result' && resultText}
			<div class="mt-2 whitespace-pre-wrap text-zinc-300">{resultText}</div>
		{/if}
	</div>
</div>
